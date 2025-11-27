type Tone = 'success' | 'warning' | 'info' | 'error';

type Props = {
  title: string;
  message: string;
  tone?: Tone;
  onClose?: () => void;
};

const toneStyles: Record<Tone, string> = {
  success: 'from-green-400/90 to-green-500/80 text-white',
  warning: 'from-amber-300/90 to-orange-400/80 text-foreground',
  info: 'from-blue-400/90 to-blue-500/80 text-white',
  error: 'from-red-400/90 to-rose-500/80 text-white'
};

function ToastMessage({ title, message, tone = 'info', onClose }: Props) {
  return (
    <div className="panel animate-fadeIn space-y-2">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-xs uppercase tracking-wide text-muted">{tone}</p>
          <h3 className="text-lg font-semibold">{title}</h3>
        </div>
        {onClose && (
          <button type="button" aria-label="Close" className="btn-ghost" onClick={onClose}>
            ×
          </button>
        )}
      </div>
      <div
        className={`rounded-soft bg-gradient-to-r px-4 py-3 text-sm shadow-soft-glow ${toneStyles[tone]} backdrop-blur-glass`}
      >
        {message}
      </div>
    </div>
  );
}

export default ToastMessage;
