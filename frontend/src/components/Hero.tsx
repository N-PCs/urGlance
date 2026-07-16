import React from 'react';
import { color, motion } from 'framer-motion';
import { Download } from 'lucide-react';
import { ProductMockup } from './ProductMockup';

export function Hero() {
  return (
    <section className="hero" id="hero">
      <div className="container">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="hero-eyebrow" style={{background:"white", color:"black"}}>
            Hybrid Rust & C++ Engine
          </div>

          <h1 className="hero-title">
            See every file
            <br />
            <span className="gradient-text">at a glance</span>
          </h1>

          <p className="hero-subtitle">
            A fast, dependency-free file organizer that previews any file the
            instant you hover — text, code, or images. Built for zero-latency
            file intelligence on every platform.
          </p>

          <motion.div
            className="hero-actions"
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.25, duration: 0.5 }}
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
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M7 17l9.2-9.2M17 17V7.8M17 7.8H7.8" />
              </svg>
            </a>
          </motion.div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.35, duration: 0.7, ease: [0.16, 1, 0.3, 1] }}
        >
          <ProductMockup />
        </motion.div>
      </div>
    </section>
  );
}

export default Hero;
