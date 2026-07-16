import React from 'react';
import { Github, Twitter, Mail } from 'lucide-react';

export function Footer() {
  return (
    <footer className="footer" id="footer">
      <div className="container">
        <div className="footer-inner">
          {/* Brand */}
          <div className="footer-brand">
            <div className="navbar-logo">
              <span className="gradient-text">          <img src="/logo.png" alt="urGlance Logo" width={70} height={40} />
urGlance</span>
            </div>
            <p>
              A high-performance file organizer for Windows &amp; Linux with instant
              preview extraction, powered by a hybrid Rust + C++ engine.
            </p>
          </div>

          {/* Links */}
          <div className="footer-links-group">
            <h4>Product</h4>
            <a href="#features">Features</a>
            <a href="#how-it-works">How It Works</a>
            <a href="#downloads">Downloads</a>
          </div>

          <div className="footer-links-group">
            <h4>Resources</h4>
            <a href="https://github.com/N-PCs/urGlance" target="_blank" rel="noreferrer">
              GitHub
            </a>
            <a href="https://github.com/N-PCs/urGlance/issues" target="_blank" rel="noreferrer">
              Report Bug
            </a>
            <a href="https://github.com/N-PCs/urGlance/releases" target="_blank" rel="noreferrer">
              Releases
            </a>
          </div>

          <div className="footer-links-group">
            <h4>Developer</h4>
            <a href="https://github.com/N-PCs" target="_blank" rel="noreferrer">
              N-PCs
            </a>
          </div>
        </div>

        <div className="footer-bottom">
          <span>&copy; {new Date().getFullYear()} urGlance. All rights reserved.</span>
          <div className="footer-socials">
            <a
              href="https://github.com/N-PCs/urGlance"
              target="_blank"
              rel="noreferrer"
              aria-label="GitHub"
            >
              <Github size={16} />
            </a>
            <a
              href="https://twitter.com"
              target="_blank"
              rel="noreferrer"
              aria-label="Twitter"
            >
              <Twitter size={16} />
            </a>
            <a href="mailto:hello@urglance.dev" aria-label="Email">
              <Mail size={16} />
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}

export default Footer;
