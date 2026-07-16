import React from 'react';
import { motion } from 'framer-motion';
import { Zap, Eye, Shield, Cpu, FileSearch, Layers } from 'lucide-react';

const features = [
  {
    icon: <Zap size={22} />,
    title: 'Sub-Millisecond Previews',
    description:
      'C++ file parser extracts text snippets, image dimensions, and metadata in under 1ms — no external dependencies needed.',
  },
  {
    icon: <Eye size={22} />,
    title: 'Hover-to-Preview',
    description:
      'Just hover over a file to see its contents instantly. Smart debouncing ensures zero wasted compute during rapid scrolling.',
  },
  {
    icon: <Shield size={22} />,
    title: 'Zero Dependencies',
    description:
      'Ships as a single static binary for Windows & Linux. No runtime, no framework, no install wizard — just drop and run.',
  },
  {
    icon: <Cpu size={22} />,
    title: 'Hybrid Rust + C++ Engine',
    description:
      'Rust handles async task management and concurrency, while C++ performs raw, low-overhead file parsing via a safe FFI bridge.',
  },
  {
    icon: <FileSearch size={22} />,
    title: 'Smart File Detection',
    description:
      'Automatically identifies 10+ file types — text, PNG, BMP, directories, and more — with graceful fallbacks for unknown formats.',
  },
  {
    icon: <Layers size={22} />,
    title: 'Task Cancellation',
    description:
      'Thread-safe hover manager instantly cancels stale preview tasks when the user moves to a new file, keeping the system responsive.',
  },
];

const cardVariants = {
  hidden: { opacity: 0, y: 30 },
  visible: (i: number) => ({
    opacity: 1,
    y: 0,
    transition: { delay: i * 0.1, duration: 0.5, ease: [0.16, 1, 0.3, 1] },
  }),
};

export function Features() {
  return (
    <section className="features" id="features">
      <div className="container">
        <div className="features-header">
          <div className="section-badge">
            <Zap size={14} />
            Features
          </div>
          <h2 className="section-title">
            Built for <span className="gradient-text">raw speed</span>
          </h2>
          <p className="section-subtitle">
            Every architectural decision in urGlance is optimized for instant
            file intelligence — from the hybrid engine to the non-blocking UI
            pipeline.
          </p>
        </div>

        <div className="features-grid">
          {features.map((feature, i) => (
            <motion.div
              key={feature.title}
              className="glass-card feature-card"
              custom={i}
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true, margin: '-60px' }}
              variants={cardVariants}
            >
              <div className="feature-icon">{feature.icon}</div>
              <h3>{feature.title}</h3>
              <p>{feature.description}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

export default Features;
