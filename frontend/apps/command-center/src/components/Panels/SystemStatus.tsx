import React from 'react';

interface SystemStatusProps {
  healthy: boolean;
  tracks: number;
  threats: number;
}

export default function SystemStatus({ healthy, tracks, threats }: SystemStatusProps) {
  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <p className="text-sm text-slate-400">System Status</p>
        <div className="flex items-center gap-2">
          <div className={`w-3 h-3 rounded-full ${healthy ? 'bg-green-500' : 'bg-red-500'}`} />
          <p className="text-sm font-semibold text-white">
            {healthy ? 'Operational' : 'Down'}
          </p>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div className="bg-slate-800 rounded p-2">
          <p className="text-xs text-slate-400">Active Tracks</p>
          <p className="text-2xl font-bold text-white">{tracks}</p>
        </div>
        <div className="bg-slate-800 rounded p-2">
          <p className="text-xs text-slate-400">Threats</p>
          <p className="text-2xl font-bold text-red-400">{threats}</p>
        </div>
      </div>

      <div className="text-xs text-slate-400 space-y-1">
        <div className="flex justify-between">
          <span>Fusion Engine:</span>
          <span className="text-green-400">⚙ 50ms</span>
        </div>
        <div className="flex justify-between">
          <span>AI Services:</span>
          <span className="text-green-400">⚙ 20ms</span>
        </div>
        <div className="flex justify-between">
          <span>Orchestrator:</span>
          <span className="text-green-400">⚙ 15ms</span>
        </div>
      </div>
    </div>
  );
}
