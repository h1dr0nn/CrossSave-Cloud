type Props = {
  formats: string[];
  value: string;
  onChange: (value: string) => void;
};

function FormatSelector({ formats, value, onChange }: Props) {
  return (
    <div className="panel flex items-center gap-2">
      <p className="text-sm font-medium text-muted">Format</p>
      <div className="flex items-center gap-2 rounded-soft bg-white/40 p-1 shadow-inner-glow backdrop-blur-glass dark:bg-white/10">
        {formats.map((format) => {
          const isActive = format === value;
          return (
            <button
              key={format}
              type="button"
              className={`rounded-soft px-3 py-1.5 text-sm font-medium transition duration-fast ${
                isActive
                  ? 'bg-accent text-white shadow-soft-glow'
                  : 'text-foreground hover:bg-white/60 dark:hover:bg-white/20'
              }`}
              onClick={() => onChange(format)}
            >
              {format.toUpperCase()}
            </button>
          );
        })}
      </div>
    </div>
  );
}

export default FormatSelector;
