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
              <svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg" width="28" height="28">
                <circle cx="16" cy="16" r="14" stroke="url(#ft-g)" strokeWidth="2.5" />
                <path
                  d="M10 20l6-12 6 12"
                  stroke="url(#ft-g)"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  fill="none"
                />
                <circle cx="16" cy="12" r="2" fill="#a78bfa" />
                <defs>
                  <linearGradient id="ft-g" x1="0" y1="0" x2="32" y2="32">
                    <stop stopColor="#a78bfa" />
                    <stop offset="1" stopColor="#06b6d4" />
                  </linearGradient>
                </defs>
              </svg>
              <span className="gradient-text">urGlance</span>
            </div>
            <p>
              A high-performance file organizer with instant preview extraction,
              powered by a hybrid Rust + C++ engine.
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
