import React from 'react';
import { motion } from 'framer-motion';
import { TubesBackground } from './TubesBackground';
import { Download, Sparkles } from 'lucide-react';

export function Hero() {
  return (
    <section className="hero" id="hero">
      {/* Interactive 3D Canvas Background */}
      <TubesBackground />

      {/* Gradient overlay for text legibility */}
      <div className="hero-gradient-overlay" />

      {/* Content */}
      <motion.div
        className="hero-content"
        initial={{ opacity: 0, y: 40 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
      >
        <motion.div
          className="hero-eyebrow"
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ delay: 0.2, duration: 0.5 }}
        >
          <span
            style={{
              width: 6,
              height: 6,
              borderRadius: '50%',
              background: '#22c55e',
              display: 'inline-block',
              boxShadow: '0 0 8px #22c55e',
            }}
          />
          Hybrid Rust + C++ Engine
        </motion.div>

        <h1 className="hero-title">
          See Every File
          <br />
          <span className="gradient-text-warm">At a Glance</span>
        </h1>

        <p className="hero-subtitle">
          A blazing-fast file organizer powered by a hybrid Rust &amp; C++ engine.
          Instant previews, zero-dependency binary, and sub-millisecond
          responsiveness — all in a single executable.
        </p>

        <motion.div
          className="hero-actions"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.5, duration: 0.6 }}
          style={{ pointerEvents: 'auto' }}
        >
          <a href="#downloads" className="btn-primary">
            <Download size={18} />
            Download Now
          </a>
          <a
            href="https://github.com/N-PCs/urGlance"
            target="_blank"
            rel="noreferrer"
            className="btn-secondary"
          >
            View on GitHub
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M7 17l9.2-9.2M17 17V7.8M17 7.8H7.8" />
            </svg>
          </a>
        </motion.div>
      </motion.div>

      {/* Bottom hint */}
      <div className="hero-hint">
        <Sparkles size={18} />
        <span>Ambient background in motion · tap to recolor</span>
      </div>
    </section>
  );
}

export default Hero;
