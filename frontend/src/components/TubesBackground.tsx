import React, { useEffect, useRef, useState } from 'react';

const randomColors = (count: number): string[] =>
  Array.from({ length: count }, () =>
    '#' + Math.floor(Math.random() * 16777215).toString(16).padStart(6, '0')
  );

interface TubesBackgroundProps {
  children?: React.ReactNode;
  className?: string;
  enableClickInteraction?: boolean;
}

export function TubesBackground({
  children,
  className,
  enableClickInteraction = true,
}: TubesBackgroundProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const tubesRef = useRef<any>(null);

  useEffect(() => {
    let mounted = true;
    let cleanup: (() => void) | undefined;
    let rafId = 0;

    const initTubes = async () => {
      if (!canvasRef.current) return;

      try {
        // Dynamic import from CDN — the threejs-components package exposes
        // a default factory for the tubes cursor effect.
        const module = await (Function(
          'return import("https://cdn.jsdelivr.net/npm/threejs-components@0.0.19/build/cursors/tubes1.min.js")'
        )() as Promise<{ default: any }>);

        const TubesCursor = module.default;

        if (!mounted) return;

        const app = TubesCursor(canvasRef.current, {
          tubes: {
            colors: ['#a78bfa', '#06b6d4', '#f472b6'],
            lights: {
              intensity: 200,
              colors: ['#a78bfa', '#06b6d4', '#f472b6', '#60aed5'],
            },
          },
        });

        tubesRef.current = app;
        setIsLoaded(true);

        // ----------------------------------------------------------------
        // Autonomous background animation.
        // The library is a *cursor* effect — it tracks the real pointer.
        // We block real pointer input (so it no longer reacts to the mouse)
        // and instead feed it synthetic pointer events along a rotating set
        // of looping patterns, so the tubes run on their own in the
        // background at all times.
        // ----------------------------------------------------------------
        const blockRealPointer = (e: PointerEvent) => {
          // Only suppress genuine (trusted) pointer movement; our synthetic
          // events stay trusted === false and pass straight through.
          if (e.isTrusted) e.stopImmediatePropagation();
        };
        window.addEventListener('pointermove', blockRealPointer, true);

        const PATTERN_DURATION = 9; // seconds each pattern stays active
        let patternIndex = 0;
        let patternStart = 0;
        let clock = 0;
        let prev = performance.now();

        const patterns = [
          // Lissajous figure-eight
          (t: number, w: number, h: number) => ({
            x: w * 0.5 + Math.sin(t * 0.6) * w * 0.32,
            y: h * 0.5 + Math.sin(t * 1.2) * h * 0.3,
          }),
          // Circular orbit
          (t: number, w: number, h: number) => ({
            x: w * 0.5 + Math.cos(t * 0.8) * w * 0.33,
            y: h * 0.5 + Math.sin(t * 0.8) * h * 0.33,
          }),
          // Diagonal sweep
          (t: number, w: number, h: number) => ({
            x: ((Math.sin(t * 0.35) + 1) / 2) * w,
            y: ((Math.cos(t * 0.5) + 1) / 2) * h,
          }),
          // Smooth wandering path
          (t: number, w: number, h: number) => ({
            x: w * (0.5 + 0.28 * Math.sin(t * 0.37) + 0.12 * Math.sin(t * 1.7)),
            y: h * (0.5 + 0.28 * Math.cos(t * 0.43) + 0.12 * Math.cos(t * 1.3)),
          }),
        ];

        const tick = (now: number) => {
          const dt = (now - prev) / 1000;
          prev = now;
          clock += dt;

          if (clock - patternStart > PATTERN_DURATION) {
            patternStart = clock;
            patternIndex = (patternIndex + 1) % patterns.length;
          }

          const w = window.innerWidth;
          const h = window.innerHeight;
          const p = patterns[patternIndex](clock, w, h);

          document.body.dispatchEvent(
            new PointerEvent('pointermove', {
              clientX: p.x,
              clientY: p.y,
              bubbles: true,
              cancelable: true,
            })
          );

          rafId = requestAnimationFrame(tick);
        };
        rafId = requestAnimationFrame(tick);

        cleanup = () => {
          window.removeEventListener('pointermove', blockRealPointer, true);
          cancelAnimationFrame(rafId);
        };
      } catch (error) {
        console.error('Failed to load TubesCursor:', error);
      }
    };

    initTubes();

    return () => {
      mounted = false;
      cleanup?.();
    };
  }, []);

  const handleClick = () => {
    if (!enableClickInteraction || !tubesRef.current) return;
    tubesRef.current.tubes.setColors(randomColors(3));
    tubesRef.current.tubes.setLightsColors(randomColors(4));
  };

  return (
    <div
      className={`hero-canvas-container ${className ?? ''}`}
      onClick={handleClick}
      style={{ position: 'absolute', inset: 0, overflow: 'hidden' }}
    >
      <canvas
        ref={canvasRef}
        className="hero-canvas"
        style={{ touchAction: 'none' }}
      />
      {/* Overlay children */}
      <div
        style={{
          position: 'relative',
          zIndex: 10,
          width: '100%',
          height: '100%',
          pointerEvents: 'none',
        }}
      >
        {children}
      </div>
    </div>
  );
}

export default TubesBackground;
