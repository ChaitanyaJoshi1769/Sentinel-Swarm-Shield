import React from 'react';

interface Threat {
  id: string;
  trackId: string;
  droneType: string;
  attackProbability: number;
  timeToImpact: number;
  priorityScore: number;
}

interface ThreatPanelProps {
  threats: Threat[];
}

export default function ThreatPanel({ threats }: ThreatPanelProps) {
  const sortedThreats = [...threats].sort((a, b) => b.priorityScore - a.priorityScore);

  return (
    <div className="p-4">
      <h2 className="text-xl font-bold text-white mb-4">Active Threats</h2>

      {sortedThreats.length === 0 ? (
        <p className="text-slate-400">No active threats</p>
      ) : (
        <div className="space-y-3">
          {sortedThreats.map((threat) => (
            <div
              key={threat.id}
              className="p-3 bg-slate-800 rounded border-l-4 border-red-500"
            >
              <div className="flex justify-between items-start mb-2">
                <div>
                  <p className="text-white font-semibold">{threat.droneType}</p>
                  <p className="text-xs text-slate-400">{threat.trackId}</p>
                </div>
                <div className="text-right">
                  <p className="text-2xl font-bold text-red-400">{threat.priorityScore}</p>
                  <p className="text-xs text-slate-400">priority</p>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-2 text-sm">
                <div>
                  <p className="text-slate-400">Attack Prob</p>
                  <p className="text-white font-semibold">
                    {Math.round(threat.attackProbability * 100)}%
                  </p>
                </div>
                <div>
                  <p className="text-slate-400">ETA</p>
                  <p className="text-white font-semibold">
                    {Math.round(threat.timeToImpact)}s
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
