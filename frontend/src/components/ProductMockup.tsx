import React from 'react';
import { Folder, FileText, Image, FileCode, HardDrive } from 'lucide-react';

const files = [
  { name: 'report.md',        type: 'txt', label: 'MD',   size: '4.2 KB', icon: <FileText size={15} />, hovered: false },
  { name: 'main.rs',          type: 'txt', label: 'RS',   size: '11.8 KB', icon: <FileCode size={15} />, hovered: false },
  { name: 'preview.cpp',      type: 'txt', label: 'CPP',  size: '8.1 KB', icon: <FileCode size={15} />, hovered: true },
  { name: 'banner.png',       type: 'img', label: 'PNG',  size: '1.4 MB', icon: <Image size={15} />, hovered: false },
  { name: 'assets/',          type: 'dir', label: 'DIR',  size: '—',      icon: <Folder size={15} />, hovered: false },
  { name: 'README.md',        type: 'txt', label: 'MD',   size: '2.0 KB', icon: <FileText size={15} />, hovered: false },
];

const previewLines: { n: number; html: React.ReactNode }[] = [
  { n: 1,  html: <><span className="kw">fn</span> extract_preview(path: &Path) <span className="kw">-&gt;</span> PreviewData {'{'}</> },
  { n: 2,  html: <>  <span className="kw">let</span> meta = read_header(&path);</> },
  { n: 3,  html: <>  <span className="kw">let</span> snippet = read_lines(&path, <span className="str">15</span>);</> },
  { n: 4,  html: <>  PreviewData {'{'}</> },
  { n: 5,  html: <>    file_type: meta.kind,</> },
  { n: 6,  html: <>    snippet,</> },
  { n: 7,  html: <>    size: meta.bytes,</> },
  { n: 8,  html: <>  {'}'}</> },
  { n: 9,  html: <>{'}'}</> },
];

export function ProductMockup() {
  return (
    <div className="mockup" role="img" aria-label="urGlance application window showing a file list and live preview pane">
      <div className="mockup-titlebar">
        <div className="mockup-dots"><span /><span /><span /></div>
        <div className="mockup-path">
          <HardDrive size={13} />
          ~/Projects/urglance
        </div>
      </div>

      <div className="mockup-body">
        {/* Sidebar */}
        <aside className="mockup-sidebar">
          <h5>Quick Access</h5>
          <div className="mockup-nav-item active"><Folder size={15} /> Projects</div>
          <div className="mockup-nav-item"><Folder size={15} /> Documents</div>
          <div className="mockup-nav-item"><Folder size={15} /> Downloads</div>
          <div className="mockup-nav-item"><Folder size={15} /> Pictures</div>
          <h5 style={{ marginTop: '1rem' }}>Drives</h5>
          <div className="mockup-nav-item"><HardDrive size={15} /> Local Disk (C:)</div>
        </aside>

        {/* File list */}
        <div className="mockup-files">
          {files.map((f) => (
            <div key={f.name} className={`mockup-file-row ${f.hovered ? 'hovered' : ''}`}>
              <span className="mockup-file-name">
                {f.icon}
                {f.name}
              </span>
              <span className={`mockup-file-type ${f.type}`}>{f.label}</span>
              <span className="mockup-file-size">{f.size}</span>
            </div>
          ))}
        </div>

        {/* Preview pane */}
        <div className="mockup-preview">
          <div className="mockup-preview-head">
            <h4>preview.cpp</h4>
            <span className="mockup-preview-badge">
              <span style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--success)', display: 'inline-block' }} />
              0.8ms
            </span>
          </div>
          <div className="mockup-preview-meta">
            <div className="mockup-meta-row"><span>type</span><span>C++ source</span></div>
            <div className="mockup-meta-row"><span>lines</span><span>218</span></div>
            <div className="mockup-meta-row"><span>encoding</span><span>UTF-8</span></div>
          </div>
          <div className="mockup-preview-code">
            {previewLines.map((l) => (
              <div key={l.n}><span className="ln">{l.n}</span>{l.html}</div>
            ))}
          </div>
        </div>
      </div>

      <div className="mockup-statusbar">
        <span><span className="ok">●</span> 6 items</span>
        <span>hover to preview</span>
        <span style={{ marginLeft: 'auto' }}>Rust · C++ · 0 deps</span>
      </div>
    </div>
  );
}

export default ProductMockup;
