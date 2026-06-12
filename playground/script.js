import init, { compile, lint, validate } from './pkg/llm_format.js?v=55';

const FILES = {
  'sentiment.llm': `agent: SentimentAnalyzer
system:
  role: classifier
  instruction: "Classify the input as positive, negative, or neutral."
user: "{input_text}"
output:
  sentiment: "one of: positive, negative, neutral"
  confidence: "float between 0 and 1"
vars:
  input_text: "The product broke on day one."
`,
  'translator.llm': `agent: Translator
system:
  role: translator
  instruction: "Translate the input text to French."
user: "{source_text}"
vars:
  source_text: "Hello, how are you today?"
`,
  'schema-validation.llm': `agent: DataExtractor
system:
  role: analyst
  instruction: "Extract the financial figures from the user message."
user: "{financial_report}"
output:
  revenue: "number (annual revenue)"
  ebitda: "number (operating profit)"
  growth: "float percentage"
vars:
  financial_report: "Our Q4 revenue was 4.2M with 1.1M EBITDA, representing 12% growth."
`
};

// Elements
const editor = document.getElementById('editor');
const gutter = document.getElementById('gutter');
const outputRaw = document.getElementById('output-raw');
const outputAst = document.getElementById('output-ast');
const problemsList = document.getElementById('problems-list');
const terminalOutput = document.getElementById('terminal-output');
const targetSelect = document.getElementById('target');
const status = document.getElementById('status');
const charCount = document.getElementById('char-count');
const problemsTabBtn = document.querySelector('[data-tab="problems"]');
const terminalTabBtn = document.querySelector('[data-tab="terminal"]');

// Preloaded active state
let activeFile = 'sentiment.llm';

// Setup file Explorer sidebar clicks
function initExplorer() {
  document.querySelectorAll('.sidebar-file').forEach(el => {
    el.addEventListener('click', (e) => {
      const filename = el.getAttribute('data-file');
      switchFile(filename);
    });
  });
}

function switchFile(filename) {
  activeFile = filename;
  document.querySelectorAll('.sidebar-file').forEach(el => {
    el.classList.remove('active');
  });
  const activeEl = document.querySelector(`[data-file="${filename}"]`);
  if (activeEl) activeEl.classList.add('active');

  const tabName = document.getElementById('editor-tab-name');
  if (tabName) tabName.textContent = filename;
  const fileLabel = document.getElementById('active-file-label');
  if (fileLabel) fileLabel.textContent = filename;

  // Load editor value
  editor.value = FILES[filename];
  updateCharCount();
  updateGutter();
  triggerLiveCompile();
}

// Update character count
function updateCharCount() {
  charCount.textContent = `${editor.value.length} chars`;
}

// Update line numbers gutter
function updateGutter() {
  const lines = editor.value.split('\n');
  const lineCount = lines.length;
  
  let html = '';
  for (let i = 1; i <= lineCount; i++) {
    html += `
      <div class="gutter-row" data-line="${i}">
        <span class="gutter-num">${i}</span>
        <span class="gutter-badge" id="gutter-badge-${i}"></span>
      </div>
    `;
  }
  gutter.innerHTML = html;
}

// Sync gutter scroll position with editor textarea
editor.addEventListener('scroll', () => {
  gutter.scrollTop = editor.scrollTop;
});

// Sync gutter heights and line updates on keyups/inputs
editor.addEventListener('input', () => {
  updateCharCount();
  updateGutter();
  triggerLiveCompile();
});

// Setup Target Selector changes
targetSelect.addEventListener('change', () => {
  triggerLiveCompile();
});

// Tabs switching logic
function initTabs() {
  // Preview Tabs (Raw Payload vs Visual AST)
  document.querySelectorAll('.preview-tab').forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.preview-tab').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      const tabTarget = tab.getAttribute('data-tab');
      if (tabTarget === 'payload') {
        outputRaw.style.display = 'block';
        outputAst.style.display = 'none';
      } else {
        outputRaw.style.display = 'none';
        outputAst.style.display = 'block';
      }
    });
  });

  // Terminal Console Tabs (Problems vs Terminal logs)
  document.querySelectorAll('.console-tab').forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.console-tab').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      const tabTarget = tab.getAttribute('data-tab');
      if (tabTarget === 'problems') {
        problemsList.style.display = 'block';
        terminalOutput.style.display = 'none';
      } else {
        problemsList.style.display = 'none';
        terminalOutput.style.display = 'block';
      }
    });
  });
}

// Set status pill state
function setStatus(text, className) {
  status.textContent = text;
  status.className = `status-pill ${className}`;
}

// Parse error lines using regex
function parseErrorLine(errorStr) {
  const match = errorStr.match(/at (\d+):(\d+)/);
  if (match) {
    return parseInt(match[1], 10);
  }
  return null;
}

// Render dynamic badges on the gutter
function renderGutterBadges(errors) {
  // Reset all badges
  document.querySelectorAll('.gutter-badge').forEach(badge => {
    badge.textContent = '';
    badge.className = 'gutter-badge';
    badge.title = '';
  });

  errors.forEach(err => {
    const lineNum = parseErrorLine(err);
    if (lineNum) {
      const badge = document.getElementById(`gutter-badge-${lineNum}`);
      if (badge) {
        badge.textContent = '✗';
        badge.className = 'gutter-badge error';
        badge.title = err;
      }
    }
  });
}

// Collapsible AST Tree Node Renderer
function renderASTNode(key, value) {
  const node = document.createElement('div');
  node.className = 'ast-node';

  if (key !== null) {
    const keySpan = document.createElement('span');
    keySpan.className = 'ast-key';
    keySpan.textContent = `"${key}": `;
    node.appendChild(keySpan);
  }

  if (value === null) {
    const valSpan = document.createElement('span');
    valSpan.className = 'ast-null';
    valSpan.textContent = 'null';
    node.appendChild(valSpan);
  } else if (typeof value === 'object') {
    const isArray = Array.isArray(value);
    const openChar = isArray ? '[' : '{';
    const closeChar = isArray ? ']' : '}';

    const toggle = document.createElement('span');
    toggle.className = 'ast-toggle expanded';
    toggle.textContent = '▼';
    node.appendChild(toggle);

    const openSpan = document.createElement('span');
    openSpan.className = 'ast-bracket';
    openSpan.textContent = openChar;
    node.appendChild(openSpan);

    const container = document.createElement('div');
    container.className = 'ast-container';

    const keys = Object.keys(value);
    keys.forEach(k => {
      const child = renderASTNode(isArray ? null : k, value[k]);
      container.appendChild(child);
    });
    node.appendChild(container);

    const closeSpan = document.createElement('span');
    closeSpan.className = 'ast-bracket';
    closeSpan.textContent = closeChar;
    node.appendChild(closeSpan);

    toggle.addEventListener('click', (e) => {
      e.stopPropagation();
      const isCollapsed = container.style.display === 'none';
      container.style.display = isCollapsed ? 'block' : 'none';
      toggle.textContent = isCollapsed ? '▼' : '▶';
      toggle.className = isCollapsed ? 'ast-toggle expanded' : 'ast-toggle collapsed';
    });
  } else {
    const valSpan = document.createElement('span');
    if (typeof value === 'string') {
      valSpan.className = 'ast-string';
      valSpan.textContent = `"${value}"`;
    } else if (typeof value === 'number') {
      valSpan.className = 'ast-number';
      valSpan.textContent = value;
    } else {
      valSpan.className = 'ast-boolean';
      valSpan.textContent = value;
    }
    node.appendChild(valSpan);
  }

  return node;
}

function buildASTTree(jsonString) {
  outputAst.innerHTML = '';
  try {
    const data = JSON.parse(jsonString);
    const tree = renderASTNode(null, data);
    outputAst.appendChild(tree);
  } catch (e) {
    outputAst.innerHTML = `<span class="ast-error">Failed to render AST: ${e.message}</span>`;
  }
}

// Live Compiler Orchestration
let debounceTimeout;
function triggerLiveCompile() {
  clearTimeout(debounceTimeout);
  debounceTimeout = setTimeout(() => {
    runCompiler();
  }, 250);
}

function runCompiler() {
  const source = editor.value.trim();
  if (!source) {
    outputRaw.textContent = 'No LLM source loaded.';
    outputAst.innerHTML = '<span class="ast-muted">No AST generated.</span>';
    problemsList.innerHTML = '<div class="problem-item clean">✓ Editor is empty</div>';
    renderGutterBadges([]);
    return;
  }

  // 1. Run Validation
  let valRes;
  try {
    valRes = JSON.parse(validate(source));
  } catch (e) {
    valRes = { ok: false, errors: [`WASM internal error: ${e.message}`] };
  }

  // 2. Run Linter
  let lintRes;
  try {
    lintRes = JSON.parse(lint(source));
  } catch (e) {
    lintRes = { warnings: [] };
  }

  const errors = valRes.errors || [];
  const warnings = lintRes.warnings || [];

  // Update Gutter Icons
  renderGutterBadges(errors);

  // Update Problems Console Panel
  if (errors.length === 0 && warnings.length === 0) {
    problemsList.innerHTML = '<div class="problem-item clean">✓ 0 problems found</div>';
    problemsTabBtn.textContent = 'Problems';
  } else {
    let html = '';
    errors.forEach(err => {
      html += `
        <div class="problem-item error">
          <span class="prob-icon">✗</span>
          <span class="prob-msg">${err}</span>
        </div>
      `;
    });
    warnings.forEach(warn => {
      html += `
        <div class="problem-item warn">
          <span class="prob-icon">⚠</span>
          <span class="prob-msg">[${warn.code}] ${warn.message}</span>
        </div>
      `;
    });
    problemsList.innerHTML = html;
    problemsTabBtn.textContent = `Problems (${errors.length + warnings.length})`;
  }

  // 3. Run Compilation Target
  const target = targetSelect.value;
  let compRes;
  try {
    compRes = JSON.parse(compile(source, target));
  } catch (e) {
    compRes = { ok: false, errors: [`WASM compile error: ${e.message}`] };
  }

  if (compRes.ok) {
    outputRaw.textContent = compRes.output;
    terminalOutput.textContent = `[${new Date().toLocaleTimeString()}] Compilation successful target=${target}`;
  } else {
    outputRaw.textContent = `Compilation failed:\n${(compRes.errors || []).join('\n')}`;
    terminalOutput.textContent = `[${new Date().toLocaleTimeString()}] Compilation failed: ${(compRes.errors || []).join('\n')}`;
  }

  // 4. Update JSON-IR Visual AST Tab
  let astRes;
  try {
    astRes = JSON.parse(compile(source, 'json-ir'));
  } catch (e) {
    astRes = { ok: false };
  }
  if (astRes.ok) {
    buildASTTree(astRes.output);
  } else {
    outputAst.innerHTML = '<span class="ast-error">Visual AST failed: Syntax errors present.</span>';
  }
}

// Initial Booting
async function boot() {
  try {
    await init('./pkg/llm_format_bg.wasm?v=55');
    setStatus('WASM ready', 'ready');
    terminalOutput.textContent = `[${new Date().toLocaleTimeString()}] WebAssembly compiler loaded successfully.`;
    initExplorer();
    initTabs();
    switchFile('sentiment.llm');
  } catch (error) {
    setStatus('WASM failed', 'error');
    outputRaw.textContent = `The in-browser compiler failed to load.\n\n${error.message}`;
    console.error(error);
  }
}

setStatus('Loading WASM…', 'loading');
boot();
