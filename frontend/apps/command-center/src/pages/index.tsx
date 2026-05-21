import React, { useState, useEffect } from 'react';
import Head from 'next/head';
import AirspaceMap from '@/components/Map/AirspaceMap';
import ThreatPanel from '@/components/Panels/ThreatPanel';
import DefenseControl from '@/components/Panels/DefenseControl';
import SystemStatus from '@/components/Panels/SystemStatus';

export default function CommandCenter() {
  const [tracks, setTracks] = useState([]);
  const [threats, setThreats] = useState([]);
  const [systemHealth, setSystemHealth] = useState(true);

  // Mock data for demo
  useEffect(() => {
    setTracks([
      {
        id: 'track-1',
        latitude: 37.7749,
        longitude: -122.4194,
        altitude: 100,
        confidence: 0.95,
        droneType: 'quadcopter',
      },
      {
        id: 'track-2',
        latitude: 37.7850,
        longitude: -122.4094,
        altitude: 150,
        confidence: 0.87,
        droneType: 'fixed_wing',
      },
    ]);

    setThreats([
      {
        id: 'threat-1',
        trackId: 'track-1',
        droneType: 'quadcopter',
        attackProbability: 0.72,
        timeToImpact: 240,
        priorityScore: 75,
      },
    ]);
  }, []);

  return (
    <>
      <Head>
        <title>Sentinel Swarm Shield - Command Center</title>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
      </Head>

      <div className="flex h-screen gap-4 bg-slate-950 p-4">
        {/* 3D Airspace Map */}
        <div className="flex-1 rounded-lg border border-slate-700 bg-slate-900 overflow-hidden">
          <AirspaceMap tracks={tracks} threats={threats} />
        </div>

        {/* Right sidebar: Threats and Controls */}
        <div className="w-96 flex flex-col gap-4">
          {/* System Status */}
          <div className="rounded-lg border border-slate-700 bg-slate-900 p-4">
            <SystemStatus healthy={systemHealth} tracks={tracks.length} threats={threats.length} />
          </div>

          {/* Threat Panel */}
          <div className="flex-1 rounded-lg border border-slate-700 bg-slate-900 overflow-y-auto">
            <ThreatPanel threats={threats} />
          </div>

          {/* Defense Control */}
          <div className="rounded-lg border border-slate-700 bg-slate-900 p-4">
            <DefenseControl threats={threats} />
          </div>
        </div>
      </div>
    </>
  );
}
