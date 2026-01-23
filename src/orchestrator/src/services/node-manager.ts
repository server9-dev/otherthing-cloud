/**
 * Node Manager Service
 *
 * Manages connected nodes, their capabilities, and health status.
 */

import { WebSocket } from 'ws';
import { v4 as uuidv4 } from 'uuid';
import {
  ConnectedNode,
  NodeCapabilities,
  NodeMessage,
  OrchestratorMessage,
  Job,
  JobRequirements,
  IPFSReadyMessage,
  WorkspaceJoinedMessage,
} from '../types/index.js';
import type { WorkspaceManager } from './workspace-manager.js';

export class NodeManager {
  private nodes: Map<string, ConnectedNode> = new Map();
  private authTokens: Map<string, string> = new Map(); // token -> nodeId
  private nodeWorkspaces: Map<string, Set<string>> = new Map(); // nodeId -> workspaceIds
  private nodeOwners: Map<string, string> = new Map(); // nodeId -> userId (owner)
  private nodeShareKeys: Map<string, string> = new Map(); // shareKey -> nodeId
  private nodeShareKeysByNode: Map<string, string> = new Map(); // nodeId -> shareKey
  private workspaceManager: WorkspaceManager | null = null;

  constructor() {
    // Start health check interval
    setInterval(() => this.healthCheck(), 30000);
  }

  /**
   * Set the workspace manager reference (called after both are initialized)
   */
  setWorkspaceManager(wm: WorkspaceManager): void {
    this.workspaceManager = wm;
  }

  /**
   * Register a new node and generate auth token + share key
   */
  registerNode(
    walletAddress: string,
    capabilities: NodeCapabilities
  ): { nodeId: string; authToken: string; shareKey: string } {
    const nodeId = capabilities.node_id || uuidv4();
    const authToken = uuidv4();

    // Generate a short, human-readable share key (8 chars)
    const shareKey = this.generateShareKey();

    this.authTokens.set(authToken, nodeId);
    this.nodeShareKeys.set(shareKey, nodeId);
    this.nodeShareKeysByNode.set(nodeId, shareKey);

    console.log(`[NodeManager] Registered node ${nodeId} with wallet ${walletAddress}, shareKey: ${shareKey}`);

    return { nodeId, authToken, shareKey };
  }

  /**
   * Generate a unique share key (8 alphanumeric chars)
   */
  private generateShareKey(): string {
    const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'; // Removed I, O, 1, 0 for clarity
    let key: string;
    do {
      key = Array.from({ length: 8 }, () => chars[Math.floor(Math.random() * chars.length)]).join('');
    } while (this.nodeShareKeys.has(key)); // Ensure uniqueness
    return key;
  }

  /**
   * Handle a new WebSocket connection from a node
   */
  handleConnection(ws: WebSocket): void {
    console.log('[NodeManager] New WebSocket connection');

    ws.on('message', (data) => {
      try {
        const message = JSON.parse(data.toString()) as NodeMessage;
        this.handleMessage(ws, message);
      } catch (error) {
        console.error('[NodeManager] Failed to parse message:', error);
        this.sendError(ws, 'PARSE_ERROR', 'Failed to parse message');
      }
    });

    ws.on('close', () => {
      this.handleDisconnect(ws);
    });

    ws.on('error', (error) => {
      console.error('[NodeManager] WebSocket error:', error);
    });
  }

  /**
   * Handle an incoming message from a node
   */
  private handleMessage(ws: WebSocket, message: NodeMessage): void {
    switch (message.type) {
      case 'register':
        this.handleRegister(ws, message);
        break;

      case 'heartbeat':
        this.handleHeartbeat(ws, message);
        break;

      case 'job_status':
        this.handleJobStatus(ws, message);
        break;

      case 'job_result':
        this.handleJobResult(ws, message);
        break;

      case 'ipfs_ready':
        this.handleIPFSReady(ws, message as unknown as IPFSReadyMessage);
        break;

      case 'ipfs_store_result':
        this.handleIPFSStoreResult(message as any);
        break;

      case 'ipfs_retrieve_result':
        this.handleIPFSRetrieveResult(message as any);
        break;

      // Sandbox operation results
      case 'sandbox_write_file_result':
      case 'sandbox_read_file_result':
      case 'sandbox_list_files_result':
      case 'sandbox_delete_file_result':
      case 'sandbox_execute_result':
      case 'sandbox_sync_ipfs_result':
      case 'sandbox_restore_ipfs_result':
      case 'pull_model_result':
      case 'llm_inference_result':
        this.handleSandboxResult(message as any);
        break;

      default:
        console.warn('[NodeManager] Unknown message type:', (message as any).type);
    }
  }

  /**
   * Handle IPFS ready message from node
   */
  private handleIPFSReady(ws: WebSocket, message: IPFSReadyMessage): void {
    const node = this.findNodeByWs(ws);
    if (!node) return;

    node.ipfsPeerId = message.peer_id;
    node.ipfsAddresses = message.addresses;
    node.ipfsReady = true;

    console.log(`[NodeManager] Node ${node.id} IPFS ready: ${message.peer_id}, ${message.addresses.length} addresses`);
  }

  private handleRegister(
    ws: WebSocket,
    message: Extract<NodeMessage, { type: 'register' }>
  ): void {
    let nodeId: string;
    let shareKey: string;
    let isReconnect = false;

    // Check if this is a returning node with auth token
    if (message.auth_token && this.authTokens.has(message.auth_token)) {
      nodeId = this.authTokens.get(message.auth_token)!;
      // Use node's provided share key, or existing, or generate new
      shareKey = (message as any).share_key || this.nodeShareKeysByNode.get(nodeId) || this.generateShareKey();
      isReconnect = true;
      console.log(`[NodeManager] Node ${nodeId} reconnected with auth token`);
    } else {
      // New registration - use node's provided ID and share key, or generate them
      nodeId = message.capabilities.node_id || uuidv4();
      // Accept share_key from node if provided (local-first architecture)
      shareKey = (message as any).share_key || this.generateShareKey();
      console.log(`[NodeManager] New node registered: ${nodeId}`);
    }

    // Clean up any old share key mappings for this node
    const oldShareKey = this.nodeShareKeysByNode.get(nodeId);
    if (oldShareKey && oldShareKey !== shareKey) {
      this.nodeShareKeys.delete(oldShareKey);
    }

    // Store share key mappings
    this.nodeShareKeys.set(shareKey.toUpperCase(), nodeId);
    this.nodeShareKeysByNode.set(nodeId, shareKey.toUpperCase());

    // Extract resource limits from registration message
    const resourceLimits = (message as any).resource_limits as {
      cpuCores?: number;
      ramPercent?: number;
      storageGb?: number;
      gpuVramPercent?: number[];
    } | undefined;

    // Extract remote control setting
    const remoteControlEnabled = (message as any).remote_control_enabled as boolean | undefined;

    // Store the connected node
    const node: ConnectedNode = {
      id: nodeId,
      capabilities: message.capabilities,
      ws,
      available: true,
      current_jobs: 0,
      last_heartbeat: new Date(),
      reputation: 50, // Start with neutral reputation
      resourceLimits,
      remoteControlEnabled: remoteControlEnabled ?? false,
    };

    this.nodes.set(nodeId, node);

    // Handle workspace assignments from node config
    const workspaceIds = (message as any).workspace_ids as string[] | undefined;
    if (workspaceIds && workspaceIds.length > 0) {
      this.nodeWorkspaces.set(nodeId, new Set(workspaceIds));
      console.log(`[NodeManager] Node ${nodeId} joined workspaces: ${workspaceIds.join(', ')}`);
    }

    // Get workspaces this node belongs to
    const assignedWorkspaces = this.nodeWorkspaces.get(nodeId);
    const workspaceList = assignedWorkspaces ? Array.from(assignedWorkspaces) : [];

    // Send registration confirmation with share key and workspaces
    this.send(ws, {
      type: 'registered',
      node_id: nodeId,
      share_key: shareKey,
      workspace_ids: workspaceList,
    });

    console.log(
      `[NodeManager] Node ${nodeId} online: ${message.capabilities.gpus.length} GPUs, ` +
      `${message.capabilities.cpu.cores} cores, ${message.capabilities.memory.total_mb}MB RAM, ` +
      `shareKey: ${shareKey}`
    );

    // Log Ollama info if present
    if (message.capabilities.ollama) {
      console.log(
        `[NodeManager] Node ${nodeId} Ollama: endpoint=${message.capabilities.ollama.endpoint}, ` +
        `models=${message.capabilities.ollama.models?.length || 0}`
      );
    } else {
      console.log(`[NodeManager] Node ${nodeId} has no Ollama configured`);
    }
  }

  private handleHeartbeat(
    ws: WebSocket,
    message: Extract<NodeMessage, { type: 'heartbeat' }>
  ): void {
    const node = this.findNodeByWs(ws);
    if (node) {
      node.available = message.available;
      node.current_jobs = message.current_jobs;
      node.last_heartbeat = new Date();
      // Update remote control setting if provided
      const remoteControlEnabled = (message as any).remote_control_enabled;
      if (typeof remoteControlEnabled === 'boolean') {
        node.remoteControlEnabled = remoteControlEnabled;
      }
    }
  }

  private handleJobStatus(
    ws: WebSocket,
    message: Extract<NodeMessage, { type: 'job_status' }>
  ): void {
    console.log(`[NodeManager] Job ${message.job_id} status: ${message.status}`);
    // TODO: Update job status in job queue
  }

  private handleJobResult(
    ws: WebSocket,
    message: Extract<NodeMessage, { type: 'job_result' }>
  ): void {
    console.log(
      `[NodeManager] Job ${message.job_id} completed: ` +
      `success=${message.result.success}, time=${message.result.execution_time_ms}ms`
    );
    // TODO: Store result and notify client
  }

  private handleDisconnect(ws: WebSocket): void {
    const node = this.findNodeByWs(ws);
    if (node) {
      console.log(`[NodeManager] Node ${node.id} disconnected`);
      this.nodes.delete(node.id);
      // Clean up workspace mappings and ownership
      this.nodeWorkspaces.delete(node.id);
      this.nodeOwners.delete(node.id);
    }
  }

  /**
   * Find a suitable node for a job based on requirements
   */
  findNodeForJob(requirements: JobRequirements): ConnectedNode | null {
    const candidates = Array.from(this.nodes.values())
      .filter((node) => this.nodeMatchesRequirements(node, requirements))
      .sort((a, b) => {
        // Sort by: availability, reputation, current load
        if (a.available !== b.available) return a.available ? -1 : 1;
        if (a.reputation !== b.reputation) return b.reputation - a.reputation;
        return a.current_jobs - b.current_jobs;
      });

    return candidates[0] || null;
  }

  /**
   * Check if a node meets job requirements
   */
  private nodeMatchesRequirements(
    node: ConnectedNode,
    requirements: JobRequirements
  ): boolean {
    const caps = node.capabilities;

    // Check GPU requirements
    if (requirements.gpu) {
      const { count, min_vram_mb, requires, preferred_vendor } = requirements.gpu;

      // Must have enough GPUs
      if (caps.gpus.length < count) return false;

      // Check VRAM
      const hasEnoughVram = caps.gpus.some((gpu) => gpu.vram_mb >= min_vram_mb);
      if (!hasEnoughVram) return false;

      // Check required compute APIs
      if (requires) {
        const hasRequired = caps.gpus.some((gpu) =>
          requires.every((req) => {
            switch (req) {
              case 'cuda': return gpu.supports.cuda;
              case 'rocm': return gpu.supports.rocm;
              case 'vulkan': return gpu.supports.vulkan;
              case 'metal': return gpu.supports.metal;
              case 'opencl': return gpu.supports.opencl;
              default: return false;
            }
          })
        );
        if (!hasRequired) return false;
      }
    }

    // Check CPU requirements
    if (requirements.cpu) {
      if (caps.cpu.cores < requirements.cpu.min_cores) return false;
      if (requirements.cpu.min_threads && caps.cpu.threads < requirements.cpu.min_threads) {
        return false;
      }
      if (requirements.cpu.required_features) {
        const hasFeatures = requirements.cpu.required_features.every((f) =>
          caps.cpu.features.includes(f)
        );
        if (!hasFeatures) return false;
      }
    }

    // Check memory requirements
    if (requirements.memory) {
      if (caps.memory.available_mb < requirements.memory.min_mb) return false;
    }

    // Check storage requirements
    if (requirements.storage) {
      if (caps.storage.available_gb < requirements.storage.min_gb) return false;
    }

    // Check MCP adapter
    if (!caps.mcp_adapters.includes(requirements.mcp_adapter)) {
      // For now, accept 'docker' as a fallback
      if (requirements.mcp_adapter !== 'docker') return false;
    }

    return true;
  }

  /**
   * Assign a job to a node
   */
  assignJob(nodeId: string, job: Job): boolean {
    const node = this.nodes.get(nodeId);
    if (!node || !node.available) return false;

    this.send(node.ws, {
      type: 'job_assignment',
      job: {
        id: job.id,
        client_id: job.client_id,
        payload: job.payload,
        timeout_seconds: 3600, // TODO: from job config
        max_cost_cents: job.requirements.max_cost_cents,
      },
    });

    node.current_jobs++;

    return true;
  }

  /**
   * Cancel a job on a node
   */
  cancelJob(nodeId: string, jobId: string): void {
    const node = this.nodes.get(nodeId);
    if (node) {
      this.send(node.ws, {
        type: 'cancel_job',
        job_id: jobId,
      });
    }
  }

  /**
   * Get all connected nodes
   */
  getNodes(): ConnectedNode[] {
    return Array.from(this.nodes.values());
  }

  /**
   * Get nodes for a specific workspace
   */
  getNodesForWorkspace(workspaceId: string): ConnectedNode[] {
    const nodes: ConnectedNode[] = [];
    for (const [nodeId, workspaceIds] of this.nodeWorkspaces) {
      if (workspaceIds.has(workspaceId)) {
        const node = this.nodes.get(nodeId);
        if (node) {
          nodes.push(node);
        }
      }
    }
    return nodes;
  }

  /**
   * Add a node to a workspace
   */
  addNodeToWorkspace(nodeId: string, workspaceId: string): boolean {
    const node = this.nodes.get(nodeId);
    if (!node) return false;

    if (!this.nodeWorkspaces.has(nodeId)) {
      this.nodeWorkspaces.set(nodeId, new Set());
    }
    this.nodeWorkspaces.get(nodeId)!.add(workspaceId);
    console.log(`[NodeManager] Node ${nodeId} joined workspace ${workspaceId}`);
    return true;
  }

  /**
   * Remove a node from a workspace
   */
  removeNodeFromWorkspace(nodeId: string, workspaceId: string): boolean {
    const workspaces = this.nodeWorkspaces.get(nodeId);
    if (!workspaces) return false;

    const removed = workspaces.delete(workspaceId);
    if (removed) {
      console.log(`[NodeManager] Node ${nodeId} left workspace ${workspaceId}`);
    }
    return removed;
  }

  /**
   * Get workspaces a node belongs to
   */
  getNodeWorkspaces(nodeId: string): string[] {
    const workspaces = this.nodeWorkspaces.get(nodeId);
    return workspaces ? Array.from(workspaces) : [];
  }

  /**
   * Get node by share key
   */
  getNodeByShareKey(shareKey: string): ConnectedNode | null {
    const nodeId = this.nodeShareKeys.get(shareKey.toUpperCase());
    if (!nodeId) return null;
    return this.nodes.get(nodeId) || null;
  }

  /**
   * Get share key for a node
   */
  getNodeShareKey(nodeId: string): string | null {
    return this.nodeShareKeysByNode.get(nodeId) || null;
  }

  /**
   * Add a node to a workspace using share key
   * Returns the node ID if successful, null if share key not found
   */
  addNodeToWorkspaceByShareKey(shareKey: string, workspaceId: string): string | null {
    const nodeId = this.nodeShareKeys.get(shareKey.toUpperCase());
    if (!nodeId) return null;

    // Check if node is still connected
    const node = this.nodes.get(nodeId);
    if (!node) return null;

    if (!this.nodeWorkspaces.has(nodeId)) {
      this.nodeWorkspaces.set(nodeId, new Set());
    }
    this.nodeWorkspaces.get(nodeId)!.add(workspaceId);
    console.log(`[NodeManager] Node ${nodeId} joined workspace ${workspaceId} via share key`);

    // Send workspace_joined message with IPFS info
    this.sendWorkspaceJoinedMessage(nodeId, workspaceId);

    return nodeId;
  }

  /**
   * Send workspace_joined message to a node with IPFS swarm key and bootstrap peers
   */
  private sendWorkspaceJoinedMessage(nodeId: string, workspaceId: string): void {
    const node = this.nodes.get(nodeId);
    if (!node || !this.workspaceManager) return;

    const ipfsInfo = this.workspaceManager.getWorkspaceIPFSInfo(workspaceId);
    if (!ipfsInfo) return;

    const bootstrapPeers = this.getBootstrapPeers(workspaceId, nodeId);

    const message: WorkspaceJoinedMessage = {
      type: 'workspace_joined',
      workspace_id: workspaceId,
      ipfs_swarm_key: ipfsInfo.swarmKey,
      bootstrap_peers: bootstrapPeers,
    };

    this.send(node.ws, message);
    console.log(`[NodeManager] Sent workspace_joined to node ${nodeId} for workspace ${workspaceId} with ${bootstrapPeers.length} bootstrap peers`);
  }

  /**
   * Get IPFS bootstrap peers for a workspace (other nodes' IPFS addresses)
   */
  getBootstrapPeers(workspaceId: string, excludeNodeId?: string): string[] {
    const peers: string[] = [];

    for (const [nodeId, workspaceIds] of this.nodeWorkspaces) {
      if (workspaceIds.has(workspaceId) && nodeId !== excludeNodeId) {
        const node = this.nodes.get(nodeId);
        if (node?.ipfsReady && node.ipfsPeerId && node.ipfsAddresses) {
          // Include addresses with peer ID appended
          for (const addr of node.ipfsAddresses) {
            // Only include routable addresses (not localhost)
            if (!addr.includes('/127.0.0.1/') && !addr.includes('/::1/')) {
              peers.push(`${addr}/p2p/${node.ipfsPeerId}`);
            }
          }
        }
      }
    }

    return peers;
  }

  /**
   * Claim ownership of an unowned node
   */
  claimNode(nodeId: string, userId: string): boolean {
    const node = this.nodes.get(nodeId);
    if (!node) return false;

    // Check if already owned
    if (this.nodeOwners.has(nodeId)) {
      return false; // Already owned by someone
    }

    this.nodeOwners.set(nodeId, userId);
    console.log(`[NodeManager] Node ${nodeId} claimed by user ${userId}`);
    return true;
  }

  /**
   * Get the owner of a node
   */
  getNodeOwner(nodeId: string): string | null {
    return this.nodeOwners.get(nodeId) || null;
  }

  /**
   * Check if a user owns a node
   */
  isNodeOwner(nodeId: string, userId: string): boolean {
    const owner = this.nodeOwners.get(nodeId);
    return owner === userId;
  }

  /**
   * Check if a node is unclaimed
   */
  isNodeUnclaimed(nodeId: string): boolean {
    return !this.nodeOwners.has(nodeId);
  }

  /**
   * Send a message to a specific node
   */
  sendToNode(nodeId: string, message: OrchestratorMessage): boolean {
    const node = this.nodes.get(nodeId);
    if (!node) return false;
    this.send(node.ws, message);
    return true;
  }

  /**
   * Update resource limits for a node
   */
  updateNodeLimits(nodeId: string, limits: {
    cpuCores?: number;
    ramPercent?: number;
    storageGb?: number;
    gpuVramPercent?: number[];
  }): boolean {
    const node = this.nodes.get(nodeId);
    if (!node) return false;

    // Send limits update to the node
    this.send(node.ws, {
      type: 'update_limits',
      limits,
    } as any);

    console.log(`[NodeManager] Sent resource limits to node ${nodeId}:`, limits);
    return true;
  }

  /**
   * Find a node for a job, optionally filtered by workspace
   */
  findNodeForJobInWorkspace(requirements: JobRequirements, workspaceId?: string): ConnectedNode | null {
    let candidates = Array.from(this.nodes.values());

    // Filter by workspace if specified
    if (workspaceId) {
      candidates = candidates.filter((node) => {
        const nodeWorkspaces = this.nodeWorkspaces.get(node.id);
        return nodeWorkspaces?.has(workspaceId) ?? false;
      });
    }

    // Filter by requirements and sort
    candidates = candidates
      .filter((node) => this.nodeMatchesRequirements(node, requirements))
      .sort((a, b) => {
        if (a.available !== b.available) return a.available ? -1 : 1;
        if (a.reputation !== b.reputation) return b.reputation - a.reputation;
        return a.current_jobs - b.current_jobs;
      });

    return candidates[0] || null;
  }

  /**
   * Get node statistics
   */
  getStats(): {
    total_nodes: number;
    available_nodes: number;
    total_gpus: number;
    total_cpu_cores: number;
    total_memory_gb: number;
  } {
    const nodes = this.getNodes();
    return {
      total_nodes: nodes.length,
      available_nodes: nodes.filter((n) => n.available).length,
      total_gpus: nodes.reduce((sum, n) => sum + n.capabilities.gpus.length, 0),
      total_cpu_cores: nodes.reduce((sum, n) => sum + n.capabilities.cpu.cores, 0),
      total_memory_gb: Math.round(
        nodes.reduce((sum, n) => sum + n.capabilities.memory.total_mb, 0) / 1024
      ),
    };
  }

  private findNodeByWs(ws: WebSocket): ConnectedNode | undefined {
    return Array.from(this.nodes.values()).find((n) => n.ws === ws);
  }

  private send(ws: WebSocket, message: OrchestratorMessage): void {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message));
    }
  }

  private sendError(ws: WebSocket, code: string, message: string): void {
    this.send(ws, { type: 'error', code, message });
  }

  private healthCheck(): void {
    const now = new Date();
    const timeout = 30000; // 30 second timeout

    for (const [nodeId, node] of this.nodes) {
      const timeSinceHeartbeat = now.getTime() - node.last_heartbeat.getTime();
      if (timeSinceHeartbeat > timeout) {
        console.log(`[NodeManager] Node ${nodeId} timed out, removing`);
        node.ws.close();
        this.nodes.delete(nodeId);
        this.nodeWorkspaces.delete(nodeId);
        this.nodeOwners.delete(nodeId);
      }
    }
  }

  // ============ IPFS Storage Operations ============

  // Track pending IPFS requests
  private ipfsRequests: Map<string, {
    type: 'store' | 'retrieve';
    resolve: (result: any) => void;
    reject: (error: Error) => void;
  }> = new Map();

  /**
   * Store content in IPFS via a node
   * Returns the CID of the stored content
   */
  async storeInIPFS(workspaceId: string, content: string | object, filename?: string): Promise<string> {
    // Find a node with IPFS ready in this workspace
    const node = this.findIPFSNodeForWorkspace(workspaceId);
    if (!node) {
      throw new Error('No IPFS-capable node available in workspace');
    }

    const requestId = `ipfs-store-${Date.now()}-${Math.random().toString(36).slice(2)}`;

    return new Promise((resolve, reject) => {
      // Store the pending request
      this.ipfsRequests.set(requestId, { type: 'store', resolve, reject });

      // Send store request to node
      const contentStr = typeof content === 'string' ? content : JSON.stringify(content);
      this.send(node.ws, {
        type: 'ipfs_store',
        request_id: requestId,
        content: contentStr,
        filename,
      } as any);

      console.log(`[NodeManager] Sent IPFS store request ${requestId} to node ${node.id}`);

      // Timeout after 60 seconds
      setTimeout(() => {
        if (this.ipfsRequests.has(requestId)) {
          this.ipfsRequests.delete(requestId);
          reject(new Error('IPFS store request timed out'));
        }
      }, 60000);
    });
  }

  /**
   * Retrieve content from IPFS via a node
   * Returns the content as a string
   */
  async retrieveFromIPFS(workspaceId: string, cid: string): Promise<string> {
    // Find a node with IPFS ready in this workspace
    const node = this.findIPFSNodeForWorkspace(workspaceId);
    if (!node) {
      throw new Error('No IPFS-capable node available in workspace');
    }

    const requestId = `ipfs-retrieve-${Date.now()}-${Math.random().toString(36).slice(2)}`;

    return new Promise((resolve, reject) => {
      // Store the pending request
      this.ipfsRequests.set(requestId, { type: 'retrieve', resolve, reject });

      // Send retrieve request to node
      this.send(node.ws, {
        type: 'ipfs_retrieve',
        request_id: requestId,
        cid,
      } as any);

      console.log(`[NodeManager] Sent IPFS retrieve request ${requestId} to node ${node.id} for CID ${cid}`);

      // Timeout after 60 seconds
      setTimeout(() => {
        if (this.ipfsRequests.has(requestId)) {
          this.ipfsRequests.delete(requestId);
          reject(new Error('IPFS retrieve request timed out'));
        }
      }, 60000);
    });
  }

  /**
   * Find a node with IPFS ready for a workspace
   */
  findIPFSNodeForWorkspace(workspaceId: string): ConnectedNode | null {
    const nodes = this.getNodesForWorkspace(workspaceId);
    return nodes.find(n => n.ipfsReady && n.available) || null;
  }

  /**
   * Handle IPFS store result from node
   */
  handleIPFSStoreResult(message: { request_id: string; success: boolean; cid?: string; error?: string }): void {
    const request = this.ipfsRequests.get(message.request_id);
    if (!request || request.type !== 'store') return;

    this.ipfsRequests.delete(message.request_id);

    if (message.success && message.cid) {
      console.log(`[NodeManager] IPFS store successful: ${message.cid}`);
      request.resolve(message.cid);
    } else {
      console.log(`[NodeManager] IPFS store failed: ${message.error}`);
      request.reject(new Error(message.error || 'IPFS store failed'));
    }
  }

  /**
   * Handle IPFS retrieve result from node
   */
  handleIPFSRetrieveResult(message: { request_id: string; success: boolean; cid?: string; content?: string; error?: string }): void {
    const request = this.ipfsRequests.get(message.request_id);
    if (!request || request.type !== 'retrieve') return;

    this.ipfsRequests.delete(message.request_id);

    if (message.success && message.content !== undefined) {
      console.log(`[NodeManager] IPFS retrieve successful: ${message.cid}`);
      request.resolve(message.content);
    } else {
      console.log(`[NodeManager] IPFS retrieve failed: ${message.error}`);
      request.reject(new Error(message.error || 'IPFS retrieve failed'));
    }
  }

  // ============ Sandbox Operations ============

  // Track pending sandbox requests
  private sandboxRequests: Map<string, {
    resolve: (result: any) => void;
    reject: (error: Error) => void;
    timeout: NodeJS.Timeout;
  }> = new Map();

  /**
   * Send a request to a node and wait for the response
   * Generic method for sandbox operations
   */
  async sendNodeRequest(nodeId: string, message: any, timeoutMs: number = 60000): Promise<any> {
    const node = this.nodes.get(nodeId);
    if (!node) {
      throw new Error(`Node ${nodeId} not found`);
    }

    const requestId = `req-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    message.request_id = requestId;

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.sandboxRequests.delete(requestId);
        reject(new Error('Request timed out'));
      }, timeoutMs);

      this.sandboxRequests.set(requestId, { resolve, reject, timeout });
      this.send(node.ws, message);
    });
  }

  /**
   * Handle sandbox operation results from nodes
   */
  private handleSandboxResult(message: { request_id: string; success: boolean; [key: string]: any }): void {
    const request = this.sandboxRequests.get(message.request_id);
    if (!request) return;

    clearTimeout(request.timeout);
    this.sandboxRequests.delete(message.request_id);

    // Return the full message (includes success, content, files, etc.)
    request.resolve(message);
  }

  /**
   * Write a file to a node's sandbox
   */
  async sandboxWriteFile(
    nodeId: string,
    workspaceId: string,
    path: string,
    content: string
  ): Promise<{ success: boolean; path?: string; error?: string }> {
    return this.sendNodeRequest(nodeId, {
      type: 'sandbox_write_file',
      workspace_id: workspaceId,
      path,
      content,
    });
  }

  /**
   * Read a file from a node's sandbox
   */
  async sandboxReadFile(
    nodeId: string,
    workspaceId: string,
    path: string
  ): Promise<{ success: boolean; content?: string; error?: string }> {
    return this.sendNodeRequest(nodeId, {
      type: 'sandbox_read_file',
      workspace_id: workspaceId,
      path,
    });
  }

  /**
   * List files in a node's sandbox
   */
  async sandboxListFiles(
    nodeId: string,
    workspaceId: string,
    path?: string
  ): Promise<{ success: boolean; files?: any[]; error?: string }> {
    return this.sendNodeRequest(nodeId, {
      type: 'sandbox_list_files',
      workspace_id: workspaceId,
      path: path || '.',
    });
  }

  /**
   * Delete a file from a node's sandbox
   */
  async sandboxDeleteFile(
    nodeId: string,
    workspaceId: string,
    path: string
  ): Promise<{ success: boolean; error?: string }> {
    return this.sendNodeRequest(nodeId, {
      type: 'sandbox_delete_file',
      workspace_id: workspaceId,
      path,
    });
  }

  /**
   * Execute a command in a node's sandbox
   */
  async sandboxExecute(
    nodeId: string,
    workspaceId: string,
    command: string,
    timeout?: number
  ): Promise<{ success: boolean; stdout: string; stderr: string; exitCode: number; error?: string }> {
    return this.sendNodeRequest(nodeId, {
      type: 'sandbox_execute',
      workspace_id: workspaceId,
      command,
      timeout: timeout || 30000,
    }, (timeout || 30000) + 5000); // Add 5s buffer for network
  }

  /**
   * Sync a node's sandbox to IPFS
   */
  async sandboxSyncToIPFS(
    nodeId: string,
    workspaceId: string
  ): Promise<{ success: boolean; cid?: string; error?: string }> {
    return this.sendNodeRequest(nodeId, {
      type: 'sandbox_sync_ipfs',
      workspace_id: workspaceId,
    }, 120000); // 2 minutes for sync
  }

  /**
   * Restore a node's sandbox from IPFS
   */
  async sandboxRestoreFromIPFS(
    nodeId: string,
    workspaceId: string,
    cid: string
  ): Promise<{ success: boolean; error?: string }> {
    return this.sendNodeRequest(nodeId, {
      type: 'sandbox_restore_ipfs',
      workspace_id: workspaceId,
      cid,
    }, 120000); // 2 minutes for restore
  }

  // ============ LLM Inference via Node ============

  /**
   * Execute LLM inference on a node's local Ollama
   * This is the proper distributed approach - the node calls its own Ollama
   */
  async llmInference(
    nodeId: string,
    request: {
      model: string;
      messages: Array<{ role: 'system' | 'user' | 'assistant'; content: string }>;
      max_tokens?: number;
      temperature?: number;
    }
  ): Promise<{
    success: boolean;
    response?: {
      content: string;
      model: string;
      tokens_used?: number;
      finish_reason?: string;
    };
    error?: string;
  }> {
    const node = this.nodes.get(nodeId);
    if (!node) {
      return { success: false, error: 'Node not found' };
    }

    if (!node.capabilities.ollama?.installed) {
      return { success: false, error: 'Node does not have Ollama installed' };
    }

    console.log(`[NodeManager] Sending LLM inference to node ${nodeId.slice(0, 8)}: model=${request.model}`);

    return this.sendNodeRequest(nodeId, {
      type: 'llm_inference',
      request_id: '', // Will be set by sendNodeRequest
      model: request.model,
      messages: request.messages,
      max_tokens: request.max_tokens || 4096,
      temperature: request.temperature || 0.7,
    }, 120000); // 2 minutes timeout for inference
  }

  /**
   * Find a node with Ollama capability for a workspace that has a specific model
   */
  findNodeWithModel(workspaceId: string, model: string): ConnectedNode | null {
    const nodes = this.getOllamaNodesForWorkspace(workspaceId);

    // First try exact match
    const exactMatch = nodes.find(n =>
      n.available && n.capabilities.ollama?.models?.some(m => m.name === model)
    );
    if (exactMatch) return exactMatch;

    // Then try family match (e.g., llama3.2:1b matches request for llama3.2:3b)
    const modelFamily = model.split(':')[0];
    const familyMatch = nodes.find(n =>
      n.available && n.capabilities.ollama?.models?.some(m => m.name.startsWith(modelFamily))
    );
    if (familyMatch) return familyMatch;

    // Return any node with Ollama as last resort
    return nodes.find(n => n.available) || null;
  }

  /**
   * Find a node with sandbox capability for a workspace
   */
  findSandboxNodeForWorkspace(workspaceId: string): ConnectedNode | null {
    const nodes = this.getNodesForWorkspace(workspaceId);
    // Prefer nodes that have storage configured (available_gb > 0 means storage is set up)
    return nodes.find(n => n.available && n.capabilities.storage?.available_gb > 0) ||
           nodes.find(n => n.available) ||
           null;
  }

  // ============ Ollama Model Management ============

  // Track pending pull requests
  private pullRequests: Map<string, {
    nodeId: string;
    model: string;
    status: 'pulling' | 'completed' | 'failed';
    progress: number;
    error?: string;
    callbacks: Array<(success: boolean, error?: string) => void>;
  }> = new Map();

  /**
   * Request a node to pull an Ollama model
   * Returns a promise that resolves when the pull completes
   */
  async pullModel(nodeId: string, model: string): Promise<{ success: boolean; error?: string }> {
    const node = this.nodes.get(nodeId);
    if (!node) {
      return { success: false, error: 'Node not found' };
    }

    if (!node.capabilities.ollama?.installed) {
      return { success: false, error: 'Node does not have Ollama installed' };
    }

    // Check if model already exists
    const hasModel = node.capabilities.ollama.models.some(m =>
      m.name === model || m.name.startsWith(model.split(':')[0])
    );
    if (hasModel) {
      return { success: true };
    }

    const requestId = `pull-${nodeId}-${model}-${Date.now()}`;

    return new Promise((resolve) => {
      // Create pull request tracker
      this.pullRequests.set(requestId, {
        nodeId,
        model,
        status: 'pulling',
        progress: 0,
        callbacks: [(success, error) => resolve({ success, error })],
      });

      // Send pull command to node
      console.log(`[NodeManager] Requesting node ${nodeId} to pull model ${model}`);
      this.send(node.ws, {
        type: 'ollama_pull',
        model,
        requestId,
      });

      // Timeout after 10 minutes
      setTimeout(() => {
        const req = this.pullRequests.get(requestId);
        if (req && req.status === 'pulling') {
          req.status = 'failed';
          req.error = 'Pull timed out';
          req.callbacks.forEach(cb => cb(false, 'Pull timed out'));
          this.pullRequests.delete(requestId);
        }
      }, 10 * 60 * 1000);
    });
  }

  /**
   * Handle pull status update from node
   */
  handlePullStatus(message: { requestId: string; status: string; progress?: number; error?: string }): void {
    const req = this.pullRequests.get(message.requestId);
    if (!req) return;

    req.progress = message.progress || 0;

    if (message.status === 'completed') {
      req.status = 'completed';
      console.log(`[NodeManager] Model ${req.model} pulled successfully to node ${req.nodeId}`);

      // Update node capabilities with new model
      const node = this.nodes.get(req.nodeId);
      if (node && node.capabilities.ollama) {
        node.capabilities.ollama.models.push({
          name: req.model,
          size: 0, // Will be updated on next heartbeat
        });
      }

      req.callbacks.forEach(cb => cb(true));
      this.pullRequests.delete(message.requestId);
    } else if (message.status === 'failed') {
      req.status = 'failed';
      req.error = message.error;
      console.log(`[NodeManager] Model pull failed: ${message.error}`);
      req.callbacks.forEach(cb => cb(false, message.error));
      this.pullRequests.delete(message.requestId);
    }
  }

  /**
   * Get nodes with Ollama for a workspace
   */
  getOllamaNodesForWorkspace(workspaceId: string): ConnectedNode[] {
    return this.getNodesForWorkspace(workspaceId)
      .filter(n => n.capabilities.ollama?.installed && n.available);
  }

  /**
   * Find the best node for running a model in a workspace
   */
  findBestNodeForModel(workspaceId: string, model: string, vramNeeded: number): ConnectedNode | null {
    const nodes = this.getOllamaNodesForWorkspace(workspaceId);

    // First, try to find a node that already has the model
    for (const node of nodes) {
      const hasModel = node.capabilities.ollama?.models.some(m =>
        m.name === model || m.name.startsWith(model.split(':')[0])
      );
      if (hasModel) {
        return node;
      }
    }

    // Otherwise, find a node with enough VRAM to pull it
    for (const node of nodes) {
      const nodeVram = node.capabilities.gpus.reduce((sum, g) => sum + g.vram_mb, 0) ||
                       node.capabilities.memory.available_mb * 0.7;
      if (nodeVram >= vramNeeded) {
        return node;
      }
    }

    return null;
  }
}
