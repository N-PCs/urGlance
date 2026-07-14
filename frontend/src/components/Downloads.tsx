import React from 'react';
import { motion } from 'framer-motion';
import { Download, Monitor, Apple, Terminal } from 'lucide-react';

const platforms = [
  {
    icon: <Monitor size={28} />,
    name: 'Windows',
    description: 'Single .exe — no installer required. Just download and run.',
    buttonText: 'Download .exe',
    fileName: 'urglance-x86_64-windows.exe',
  },
  {
    icon: <Apple size={28} />,
    name: 'macOS',
    description: 'Universal binary for Apple Silicon and Intel Macs.',
    buttonText: 'Download .dmg',
    fileName: 'urglance-universal-macos.dmg',
  },
  {
    icon: <Terminal size={28} />,
    name: 'Linux',
    description: 'Available as .deb, .rpm, and a static musl binary.',
    buttonText: 'Download .tar.gz',
    fileName: 'urglance-x86_64-linux.tar.gz',
  },
];

const cardVariants = {
  hidden: { opacity: 0, y: 30 },
  visible: (i: number) => ({
    opacity: 1,
    y: 0,
    transition: { delay: i * 0.12, duration: 0.5, ease: [0.16, 1, 0.3, 1] },
  }),
};

export function Downloads() {
  return (
    <section className="downloads" id="downloads">
      <div className="container">
        <div className="downloads-header">
          <div className="section-badge">
            <Download size={14} />
            Downloads
          </div>
          <h2 className="section-title">
            Get <span className="gradient-text">urGlance</span>
          </h2>
          <p className="section-subtitle">
            One binary — every platform. No runtime dependencies, no install
            wizard, no bloat. Just drop it and go.
          </p>
        </div>

        <div className="downloads-grid">
          {platforms.map((platform, i) => (
            <motion.div
              key={platform.name}
              className="glass-card download-card"
              custom={i}
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true, margin: '-40px' }}
              variants={cardVariants}
            >
              <div className="download-icon">{platform.icon}</div>
              <h3>{platform.name}</h3>
              <p>{platform.description}</p>
              <a
                href={`https://github.com/N-PCs/urGlance/releases/latest/download/${platform.fileName}`}
                className="btn-primary"
              >
                <Download size={16} />
                {platform.buttonText}
              </a>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

export default Downloads;
