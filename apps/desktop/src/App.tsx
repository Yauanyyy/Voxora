import "./styles.css";

export function App() {
  return (
    <main className="shell" aria-labelledby="app-title">
      <div className="shell__eyebrow">M2 workspace skeleton</div>
      <h1 id="app-title">Voxora</h1>
      <p className="shell__message">
        The desktop foundation is ready. Dictation features will be added in a
        later milestone.
      </p>
      <p className="shell__status" role="status">
        No session is running.
      </p>
    </main>
  );
}
