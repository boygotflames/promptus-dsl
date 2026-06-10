import init, { compile, lint, validate } from './pkg/llm_format.js';

const EXAMPLE = `agent: SentimentAnalyzer
system:
  role: classifier
  instruction: "Classify the input as positive, negative, or neutral."
user: "{input_text}"
output:
  sentiment: "one of: positive, negative, neutral"
  confidence: "float between 0 and 1"
vars:
  input_text: "The product broke on day one."
`;

const editor = document.getElementById('editor');
const output = document.getElementById('output');
const diagnostics = document.getElementById('diagnostics');
const outputLabel = document.getElementById('output-label');
const targetSelect = document.getElementById('target');
const status = document.getElementById('status');
const charCount = document.getElementById('char-count');
const compileButton = document.getElementById('btn-compile');
const validateButton = document.getElementById('btn-validate');
const lintButton = document.getElementById('btn-lint');
const exampleButton = document.getElementById('btn-example');

function setStatus(text, className) {
  status.textContent = text;
  status.className = `status-pill ${className}`;
}

function clearDiagnostics() {
  diagnostics.innerHTML = '';
  diagnostics.style.display = 'none';
}

function showOutput(text, className) {
  output.textContent = text;
  output.className = `output-panel ${className}`;
  clearDiagnostics();
}

function showDiagnostics(items, className) {
  diagnostics.innerHTML = items
    .map(
      (item) => `
        <div class="diag-item ${className}">
          <strong>${className === 'warn' ? 'WARN' : 'ERROR'}</strong>
          <span>${item}</span>
        </div>
      `,
    )
    .join('');
  diagnostics.style.display = 'block';
}

function updateCharCount() {
  charCount.textContent = `${editor.value.length} chars`;
}

function safeParse(fn) {
  try {
    return JSON.parse(fn());
  } catch (error) {
    return {
      ok: false,
      errors: [`Internal error: ${error.message}`],
    };
  }
}

function setLoadedExample() {
  editor.value = EXAMPLE;
  updateCharCount();
}

editor.addEventListener('input', updateCharCount);

exampleButton.addEventListener('click', () => {
  setLoadedExample();
  editor.focus();
});

compileButton.addEventListener('click', () => {
  const source = editor.value.trim();
  if (!source) {
    showOutput('No .llm source loaded.', 'err');
    return;
  }

  const target = targetSelect.value;
  outputLabel.textContent = target;
  const result = safeParse(() => compile(source, target));

  if (result.ok) {
    showOutput(result.output, 'ok');
    return;
  }

  showOutput('Compilation failed.', 'err');
  showDiagnostics(result.errors || ['Unknown compile failure.'], 'error');
});

validateButton.addEventListener('click', () => {
  const source = editor.value.trim();
  if (!source) {
    showOutput('No .llm source loaded.', 'err');
    return;
  }

  outputLabel.textContent = 'validate';
  const result = safeParse(() => validate(source));

  if (result.ok) {
    showOutput('✓ valid', 'ok');
    return;
  }

  showOutput('✗ invalid', 'err');
  showDiagnostics(result.errors || ['Unknown validation failure.'], 'error');
});

lintButton.addEventListener('click', () => {
  const source = editor.value.trim();
  if (!source) {
    showOutput('No .llm source loaded.', 'err');
    return;
  }

  outputLabel.textContent = 'lint';
  const result = safeParse(() => lint(source));
  const warnings = result.warnings || [];

  if (warnings.length === 0) {
    showOutput('✓ lint clean', 'ok');
    return;
  }

  const messages = warnings.map((warning) => `[${warning.code}] ${warning.message}`);
  showOutput(messages.join('\n'), 'warn');
  showDiagnostics(messages, 'warn');
});

async function boot() {
  try {
    await init();
    setStatus('WASM ready', 'ready');
    compileButton.disabled = false;
    validateButton.disabled = false;
    lintButton.disabled = false;
    setLoadedExample();
  } catch (error) {
    setStatus('WASM failed to load', 'error');
    showOutput(
      `The in-browser compiler could not load.\n\n${error.message}`,
      'err',
    );
    console.error(error);
  }
}

setStatus('Loading WASM…', 'loading');
boot();
