import React from 'react';
import { motion } from 'framer-motion';
import { Download, Monitor, Terminal } from 'lucide-react';

const platforms = [
  {
    icon: <Monitor size={28} />,
    name: 'Windows',
    description: 'Portable archive. Extract and run the executable.',
    options: [
      { text: 'Download .zip', file: '/downloads/urglance-windows.zip' }
    ]
  },
  {
    icon: <Terminal size={28} />,
    name: 'Linux',
    description: 'Available as a static binary or standard system packages.',
    options: [
      { text: '.tar.gz', file: '/downloads/urglance-linux.tar.gz' },
      { text: '.deb (Ubuntu/Debian)', file: '/downloads/urglance-linux.deb' },
      { text: '.rpm (Fedora/RHEL)', file: '/downloads/urglance-linux.rpm' }
    ]
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
            One binary — Windows &amp; Linux only. No runtime dependencies, no
            install wizard, no bloat. Just drop it and go.
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
              <div className="download-options" style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', width: '100%', marginTop: 'auto' }}>
                {platform.options.map((opt) => (
                  <a
                    key={opt.text}
                    href={opt.file}
                    download
                    className="btn-primary"
                    style={{ width: '100%', justifyContent: 'center' }}
                  >
                    <Download size={16} />
                    {opt.text}
                  </a>
                ))}
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

export default Downloads;
