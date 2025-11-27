type Props = {
  label: string;
  progress: number;
  status?: string;
  eta?: string;
};

function ProgressIndicator({ label, progress, status, eta }: Props) {
  const safeProgress = Math.min(100, Math.max(0, progress));
  return (
    <div className="w-full space-y-2">
      <div className="flex items-center justify-between text-sm text-muted">
        <span className="font-medium text-foreground">{label}</span>
        <span>{safeProgress}%</span>
      </div>
      <div className="h-3 overflow-hidden rounded-full bg-white/40 shadow-inner-glow backdrop-blur-glass dark:bg-white/10">
        <div
          className="h-full rounded-full bg-gradient-to-r from-accent/90 via-accent to-accent/80 shadow-soft-glow transition-[width] duration-ui"
          style={{ width: `${safeProgress}%` }}
        />
      </div>
      {(status || eta) && (
        <div className="flex items-center gap-2 text-xs text-muted">
          {status && <span className="rounded-full bg-white/40 px-2 py-1 backdrop-blur-glass dark:bg-white/10">{status}</span>}
          {eta && <span className="text-foreground">ETA {eta}</span>}
        </div>
      )}
    </div>
  );
}

export default ProgressIndicator;
