import React, { useState } from 'react';

interface Threat {
  id: string;
  trackId: string;
  droneType: string;
}

interface DefenseControlProps {
  threats: Threat[];
}

export default function DefenseControl({ threats }: DefenseControlProps) {
  const [selectedThreat, setSelectedThreat] = useState<string | null>(null);

  const handleEngage = (threatId: string) => {
    // In production, would call GraphQL mutation
    console.log(`Engaging threat: ${threatId}`);
    alert(`Engagement approved for ${threatId}. Interceptor deployed.`);
  };

  return (
    <div className="space-y-4">
      <h3 className="text-lg font-bold text-white">Defense Controls</h3>

      <select
        value={selectedThreat || ''}
        onChange={(e) => setSelectedThreat(e.target.value || null)}
        className="w-full px-3 py-2 bg-slate-800 text-white rounded border border-slate-600"
      >
        <option value="">Select threat...</option>
        {threats.map((threat) => (
          <option key={threat.id} value={threat.id}>
            {threat.droneType} ({threat.trackId})
          </option>
        ))}
      </select>

      <button
        onClick={() => selectedThreat && handleEngage(selectedThreat)}
        disabled={!selectedThreat}
        className="w-full py-2 bg-red-600 text-white font-bold rounded hover:bg-red-700 disabled:bg-slate-600 disabled:cursor-not-allowed"
      >
        ENGAGE THREAT
      </button>

      <div className="space-y-2 text-sm">
        <label className="flex items-center gap-2 text-slate-300">
          <input type="checkbox" defaultChecked className="w-4 h-4" />
          Jamming active
        </label>
        <label className="flex items-center gap-2 text-slate-300">
          <input type="checkbox" defaultChecked className="w-4 h-4" />
          Interceptor ready
        </label>
      </div>
    </div>
  );
}
