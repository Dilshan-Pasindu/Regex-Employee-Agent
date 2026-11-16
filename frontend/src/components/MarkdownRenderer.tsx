// =============================================================================
// components/MarkdownRenderer.tsx
// Renders bot message content as Markdown with code-block copy buttons.
// No heavy deps — uses a lightweight DIY parser for the common cases.
// =============================================================================

import { useState } from 'react';

interface Props {
  content: string;
}

function CodeBlock({ code, lang }: { code: string; lang: string }) {
  const [copied, setCopied] = useState(false);

  function handleCopy() {
    navigator.clipboard.writeText(code).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    });
  }

  return (
    <div className="md-code-block">
      {lang && (
        <span style={{
          position: 'absolute', top: 8, left: 12,
          fontSize: '0.6rem', color: '#9b89c4', textTransform: 'uppercase', letterSpacing: 1,
        }}>
          {lang}
        </span>
      )}
      <button className="copy-btn" onClick={handleCopy} aria-label="Copy code">
        {copied ? '✓ Copied' : 'Copy'}
      </button>
      <pre><code>{code}</code></pre>
    </div>
  );
}

// Split content into code blocks and text segments
function parseSegments(content: string) {
  const segments: Array<{ type: 'text' | 'code'; content: string; lang: string }> = [];
  const codeBlockRe = /```(\w*)\n?([\s\S]*?)```/g;
  let last = 0;
  let m: RegExpExecArray | null;

  while ((m = codeBlockRe.exec(content)) !== null) {
    if (m.index > last) {
      segments.push({ type: 'text', content: content.slice(last, m.index), lang: '' });
    }
    segments.push({ type: 'code', content: m[2].trimEnd(), lang: m[1] || '' });
    last = m.index + m[0].length;
  }
  if (last < content.length) {
    segments.push({ type: 'text', content: content.slice(last), lang: '' });
  }
  return segments;
}

// Render inline markdown (bold, italic, inline code)
function InlineMarkdown({ text }: { text: string }) {
  // Split on **bold**, *italic*, `code`
  const parts = text.split(/(\*\*[^*]+\*\*|\*[^*]+\*|`[^`]+`)/g);
  return (
    <>
      {parts.map((p, i) => {
        if (p.startsWith('**') && p.endsWith('**')) {
          return <strong key={i}>{p.slice(2, -2)}</strong>;
        }
        if (p.startsWith('*') && p.endsWith('*')) {
          return <em key={i}>{p.slice(1, -1)}</em>;
        }
        if (p.startsWith('`') && p.endsWith('`')) {
          return (
            <code key={i} style={{
              background: 'rgba(124,58,237,0.15)',
              color: 'var(--g-from)',
              borderRadius: 4,
              padding: '1px 5px',
              fontFamily: 'monospace',
              fontSize: '0.85em',
            }}>
              {p.slice(1, -1)}
            </code>
          );
        }
        return <span key={i}>{p}</span>;
      })}
    </>
  );
}

function TextSegment({ content }: { content: string }) {
  const lines = content.split('\n');
  const elements: React.ReactNode[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    // Heading
    const h3 = line.match(/^###\s+(.*)/);
    const h2 = line.match(/^##\s+(.*)/);
    const h1 = line.match(/^#\s+(.*)/);
    if (h3) {
      elements.push(<h3 key={i} style={{ fontSize: '0.95rem', fontWeight: 700, margin: '8px 0 2px' }}><InlineMarkdown text={h3[1]} /></h3>);
    } else if (h2) {
      elements.push(<h2 key={i} style={{ fontSize: '1rem', fontWeight: 700, margin: '10px 0 4px' }}><InlineMarkdown text={h2[1]} /></h2>);
    } else if (h1) {
      elements.push(<h1 key={i} style={{ fontSize: '1.1rem', fontWeight: 800, margin: '10px 0 4px' }}><InlineMarkdown text={h1[1]} /></h1>);
    } else if (/^[-*]\s+/.test(line)) {
      // Collect list items
      const listItems: string[] = [];
      while (i < lines.length && /^[-*]\s+/.test(lines[i])) {
        listItems.push(lines[i].replace(/^[-*]\s+/, ''));
        i++;
      }
      elements.push(
        <ul key={`ul-${i}`} style={{ margin: '4px 0', paddingLeft: '1.2rem', listStyle: 'disc' }}>
          {listItems.map((it, j) => (
            <li key={j} style={{ marginBottom: 2 }}><InlineMarkdown text={it} /></li>
          ))}
        </ul>
      );
      continue;
    } else if (line.trim() === '') {
      elements.push(<br key={`br-${i}`} />);
    } else {
      elements.push(<p key={i} style={{ margin: '2px 0' }}><InlineMarkdown text={line} /></p>);
    }
    i++;
  }
  return <>{elements}</>;
}

export function MarkdownRenderer({ content }: Props) {
  const segments = parseSegments(content);
  return (
    <div style={{ fontSize: '0.875rem', lineHeight: 1.65, wordBreak: 'break-word' }}>
      {segments.map((seg, idx) =>
        seg.type === 'code'
          ? <CodeBlock key={idx} code={seg.content} lang={seg.lang} />
          : <TextSegment key={idx} content={seg.content} />
      )}
    </div>
  );
}
