# ADR-006: Use Next.js 15 for Command Center UI

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team

## Context

The command center UI must:
- Render 500+ tracks in real-time on 3D globe
- Update UI within 50ms of data arrival
- Support multi-operator simultaneous control
- Work offline with eventual consistency
- Deploy as static asset or SSR

**Candidates**:
1. **Next.js 15**: SSR, real-time, excellent DX
2. **React SPA**: Simple, but no SSR benefits
3. **Vue.js**: Good, but smaller ecosystem
4. **Svelte**: Fast, but smaller community

## Decision

Use **Next.js 15** with **Cesium.js** for 3D airspace visualization.

**Stack**:
- **Framework**: Next.js 15 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS + custom tactical theme
- **Maps**: Cesium.js (3D globe) + Mapbox GL
- **Real-time**: GraphQL subscriptions + WebSocket
- **State**: React Query + Zustand

## Rationale

### 1. Server-Side Rendering
- Fast first paint (<2s on 3G)
- SEO-friendly (for documentation)
- Reduced JavaScript bundle
- Better accessibility

### 2. Real-Time Capabilities
- Native WebSocket support
- GraphQL subscriptions via apolloSubscriptionsLink
- Server-Sent Events (SSE) as fallback
- Optimistic updates + conflict resolution

### 3. Performance
- Automatic code splitting
- Image optimization (next/image)
- Font optimization (next/font)
- Route-level data fetching

### 4. Developer Experience
- Fast refresh during development
- TypeScript support built-in
- Integrated API routes (/api/*)
- Middleware for auth, logging

### 5. Deployment Flexibility
- Deploy as static site (next export)
- Deploy with server (Vercel, self-hosted)
- Edge runtime support (Cloudflare Workers)
- Container-ready

## Example: Real-Time Track Subscription

```typescript
// pages/command-center.tsx
import { useSubscription } from '@apollo/client';
import { gql } from '@apollo/client';
import { AirspaceMap } from '@/components/Map/AirspaceMap';

const TRACK_SUBSCRIPTION = gql`
  subscription onTrackUpdated {
    trackUpdated {
      id
      latitude
      longitude
      altitude
      velocity {
        north
        east
        down
      }
      confidence
    }
  }
`;

export default function CommandCenter() {
  const { data, loading, error } = useSubscription(TRACK_SUBSCRIPTION);
  const [tracks, setTracks] = React.useState<Track[]>([]);
  
  React.useEffect(() => {
    if (data?.trackUpdated) {
      setTracks(prev => {
        const idx = prev.findIndex(t => t.id === data.trackUpdated.id);
        if (idx >= 0) {
          // Update existing
          const updated = [...prev];
          updated[idx] = data.trackUpdated;
          return updated;
        } else {
          // Add new
          return [...prev, data.trackUpdated];
        }
      });
    }
  }, [data]);
  
  return (
    <div className="flex h-screen gap-4 bg-slate-950 p-4">
      <div className="flex-1">
        <AirspaceMap tracks={tracks} />
      </div>
      <div className="w-80">
        <ThreatPanel threats={data?.threats || []} />
      </div>
    </div>
  );
}
```

## UI Architecture

```
command-center/
├── src/
│   ├── pages/
│   │   ├── index.tsx           # Main command center
│   │   ├── analyst.tsx         # Analyst console
│   │   ├── api/
│   │   │   └── graphql.ts      # GraphQL endpoint
│   │   └── _app.tsx            # App wrapper
│   ├── components/
│   │   ├── Map/
│   │   │   ├── AirspaceMap.tsx # Cesium globe
│   │   │   ├── TrackOverlay.tsx
│   │   │   └── ThreatCone.tsx
│   │   ├── Panels/
│   │   │   ├── ThreatPanel.tsx
│   │   │   └── EngagementStatus.tsx
│   │   └── UI/
│   │       ├── Button.tsx      # Reusable components
│   │       └── Gauge.tsx       # Tactical gauges
│   ├── hooks/
│   │   ├── useAirspace.ts      # GraphQL hooks
│   │   └── useEngagement.ts
│   ├── styles/
│   │   └── globals.css         # Tailwind + theme
│   └── lib/
│       ├── apollo.ts           # GraphQL client
│       └── geo.ts              # Geospatial math
└── package.json
```

## Dark Tactical Theme

```css
/* globals.css */
@layer base {
  :root {
    /* Tactical dark mode */
    --color-bg-primary: #0f172a;    /* Navy black */
    --color-bg-secondary: #1e293b;  /* Slate */
    --color-text-primary: #e2e8f0;  /* Light slate */
    --color-threat-high: #ef4444;   /* Red */
    --color-threat-medium: #f59e0b; /* Amber */
    --color-threat-low: #10b981;    /* Green */
  }
}

body {
  background-color: var(--color-bg-primary);
  color: var(--color-text-primary);
  font-family: "IBM Plex Mono", monospace;
}

.threat-cone-high {
  fill: var(--color-threat-high);
  opacity: 0.3;
  stroke: var(--color-threat-high);
  stroke-width: 1px;
}
```

## Performance Optimization

### Rendering 500+ Tracks
```typescript
// Use canvas rendering (not DOM) for performance
import { Canvas } from 'react-three-fiber';

<Canvas camera={{ position: [0, 0, 100] }}>
  {tracks.map(track => (
    <TrackMarker key={track.id} track={track} />
  ))}
</Canvas>
```

### Data Fetching
```typescript
// Server-side data fetching
export async function getServerSideProps() {
  const threats = await fetchThreats();
  return { props: { threats }, revalidate: 1 };
}
```

## Tradeoffs

### Advantages
- SSR for fast initial load
- Real-time subscriptions native
- Large ecosystem (UI libraries)
- Type-safe with TypeScript
- Deploy anywhere (serverless to self-hosted)

### Disadvantages
- Larger bundle than SPA
- Node.js runtime required (unless static export)
- Learning curve for App Router
- GraphQL subscriptions add complexity

## Mitigation

### Bundle Size
```javascript
// next.config.js
module.exports = {
  compress: true,
  experimental: {
    optimizePackageImports: ["@apollo/client"],
  }
};
```

### Real-Time Reliability
- Automatic reconnection on disconnect
- Local cache with React Query
- Conflict resolution via timestamps
- Fallback to polling if WebSocket fails

## Consequences

- Fast, responsive command center UI
- Real-time collaboration between operators
- 50ms update latency achievable
- Easy to extend with new views

## Related Decisions

- [ADR-009: GraphQL API Gateway](ADR-009-graphql-api-gateway.md)

## References

- [Next.js 15 Documentation](https://nextjs.org/docs)
- [Apollo Client Real-Time](https://www.apollographql.com/docs/react/real-time/subscriptions/)
- [Cesium.js Developer Guide](https://cesium.com/learn/)
