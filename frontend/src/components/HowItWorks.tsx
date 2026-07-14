import React from 'react';
import { motion } from 'framer-motion';
import { Workflow } from 'lucide-react';

const steps = [
  {
    num: '01',
    title: 'Hover Detected',
    description:
      'The UI captures a file hover event and dispatches it to the Rust-powered HoverPreviewManager. If a previous task is still running, it is instantly aborted.',
    tag: 'manager.handle_hover(path)',
  },
  {
    num: '02',
    title: '250ms Debounce Window',
    description:
      'A smart 250ms debounce timer starts. If the user moves to another file before it expires, the timer resets — ensuring zero wasted compute during rapid scrolling.',
    tag: 'tokio::time::sleep(250ms)',
  },
  {
    num: '03',
    title: 'C++ Parser Dispatch',
    description:
      'Once debounce passes, the Rust async runtime offloads the file to the C++ parser on a dedicated blocking thread pool via a safe cxx FFI bridge.',
    tag: 'ffi::extract_file_preview()',
  },
  {
    num: '04',
    title: 'Instant Preview Render',
    description:
      'The C++ engine reads file headers, extracts text snippets (capped at 15 lines), parses image dimensions, and returns structured PreviewData — all in under 1ms.',
    tag: 'PreviewData { file_type, snippet, meta }',
  },
];

const stepVariants = {
  hidden: { opacity: 0, x: -30 },
  visible: (i: number) => ({
    opacity: 1,
    x: 0,
    transition: { delay: i * 0.15, duration: 0.6, ease: [0.16, 1, 0.3, 1] },
  }),
};

export function HowItWorks() {
  return (
    <section className="how-it-works" id="how-it-works">
      <div className="container">
        <div className="how-it-works-header">
          <div className="section-badge">
            <Workflow size={14} />
            Architecture
          </div>
          <h2 className="section-title">
            How <span className="gradient-text">it works</span>
          </h2>
          <p className="section-subtitle">
            A four-stage pipeline from hover to rendered preview — designed for
            zero latency and maximum responsiveness.
          </p>
        </div>

        <div className="steps-container">
          {steps.map((step, i) => (
            <motion.div
              key={step.num}
              className="step-item"
              custom={i}
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true, margin: '-40px' }}
              variants={stepVariants}
            >
              <div className="step-number">{step.num}</div>
              <div className="step-content">
                <h3>{step.title}</h3>
                <p>{step.description}</p>
                <span className="code-tag">{step.tag}</span>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}

export default HowItWorks;
