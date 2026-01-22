import { Activity, Workflow, Bot, Server } from 'lucide-react';
import { Link } from 'react-router-dom';

export function Dashboard() {
  return (
    <div className="fade-in">
      <div className="cyber-card" style={{ marginBottom: '1rem' }}>
        <div className="cyber-card-header">
          <span className="cyber-card-title">
            <Activity size={14} style={{ marginRight: '0.5rem', verticalAlign: 'middle' }} />
            WELCOME TO OTHERTHING
          </span>
        </div>
        <div className="cyber-card-body" style={{ padding: '2rem', textAlign: 'center' }}>
          <p style={{ color: 'var(--text-secondary)', marginBottom: '2rem' }}>
            Build AI workflows and deploy agents with MCP adapters
          </p>

          <div style={{ display: 'flex', gap: '1rem', justifyContent: 'center', flexWrap: 'wrap' }}>
            <Link to="/flows" className="cyber-button" style={{ textDecoration: 'none' }}>
              <Workflow size={16} style={{ marginRight: '0.5rem' }} />
              Flow Builder
            </Link>
            <Link to="/workspaces" className="cyber-button" style={{ textDecoration: 'none' }}>
              <Bot size={16} style={{ marginRight: '0.5rem' }} />
              Workspaces & Agents
            </Link>
          </div>
        </div>
      </div>

      <div className="cyber-grid-layout">
        <div className="cyber-card">
          <div className="cyber-card-header">
            <span className="cyber-card-title">
              <Workflow size={14} style={{ marginRight: '0.5rem', verticalAlign: 'middle' }} />
              FLOW BUILDER
            </span>
          </div>
          <div className="cyber-card-body">
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
              Create visual workflows connecting MCP adapters for LLM inference, memory, tools, search, and trading.
            </p>
          </div>
        </div>

        <div className="cyber-card">
          <div className="cyber-card-header">
            <span className="cyber-card-title">
              <Bot size={14} style={{ marginRight: '0.5rem', verticalAlign: 'middle' }} />
              AGENTS
            </span>
          </div>
          <div className="cyber-card-body">
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
              Deploy autonomous agents within workspaces. Monitor their activity and progress in real-time.
            </p>
          </div>
        </div>

        <div className="cyber-card">
          <div className="cyber-card-header">
            <span className="cyber-card-title">
              <Server size={14} style={{ marginRight: '0.5rem', verticalAlign: 'middle' }} />
              MCP ADAPTERS
            </span>
          </div>
          <div className="cyber-card-body">
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
              Connect to LLM Inference, Agent orchestration, Memory storage, Tool execution, Search, and Trading services.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
