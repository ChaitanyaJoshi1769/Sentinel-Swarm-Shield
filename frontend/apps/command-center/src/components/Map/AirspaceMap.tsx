import React, { useEffect, useRef } from 'react';
import * as Cesium from 'cesium';

interface Track {
  id: string;
  latitude: number;
  longitude: number;
  altitude: number;
  confidence: number;
  droneType: string;
}

interface Threat {
  id: string;
  trackId: string;
  attackProbability: number;
  timeToImpact: number;
}

interface AirspaceMapProps {
  tracks: Track[];
  threats: Threat[];
}

export default function AirspaceMap({ tracks, threats }: AirspaceMapProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewerRef = useRef<Cesium.Viewer | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // Initialize Cesium viewer
    Cesium.Ion.defaultAccessToken = 'your-token-here'; // Replace with real token

    const viewer = new Cesium.Viewer(containerRef.current, {
      terrainProvider: Cesium.createWorldTerrain(),
      imageryProvider: Cesium.ArcGisMapServerImageryProvider.fromUrl(
        'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer'
      ),
      animation: false,
      timeline: false,
    });

    // Set default view (San Francisco)
    viewer.camera.setView({
      destination: Cesium.Cartesian3.fromDegrees(-122.4194, 37.7749, 50000),
    });

    viewerRef.current = viewer;

    return () => {
      viewer.destroy();
    };
  }, []);

  // Update track visualization
  useEffect(() => {
    if (!viewerRef.current) return;

    const viewer = viewerRef.current;

    // Clear existing entities
    viewer.entities.removeAll();

    // Add track markers
    tracks.forEach((track) => {
      const color = track.droneType === 'quadcopter'
        ? Cesium.Color.YELLOW
        : Cesium.Color.RED;

      viewer.entities.add({
        position: Cesium.Cartesian3.fromDegrees(
          track.longitude,
          track.latitude,
          track.altitude
        ),
        point: {
          pixelSize: 8,
          color: color,
          outlineColor: Cesium.Color.WHITE,
          outlineWidth: 1,
        },
        label: {
          text: `${track.id.split('-')[1]} (${Math.round(track.confidence * 100)}%)`,
          font: '10px sans-serif',
          fillColor: Cesium.Color.WHITE,
          pixelOffset: new Cesium.Cartesian2(0, -12),
        },
      });
    });

    // Add threat cones
    threats.forEach((threat) => {
      const track = tracks.find(t => t.id === threat.trackId);
      if (!track) return;

      viewer.entities.add({
        position: Cesium.Cartesian3.fromDegrees(
          track.longitude,
          track.latitude,
          track.altitude
        ),
        cylinder: {
          length: threat.timeToImpact * 10, // Simplified visualization
          topRadius: 1000,
          bottomRadius: 500,
          material: Cesium.Color.RED.withAlpha(0.2),
          outline: true,
          outlineColor: Cesium.Color.RED,
        },
      });
    });
  }, [tracks, threats]);

  return (
    <div ref={containerRef} className="w-full h-full" />
  );
}
