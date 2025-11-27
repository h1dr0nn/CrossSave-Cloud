type Props = {
  currentPath: string;
  accentLabel?: string;
  onChoose: () => void;
};

function OutputFolderChooser({ currentPath, accentLabel = 'Browse', onChoose }: Props) {
  return (
    <div className="panel flex items-center gap-3">
      <div className="flex-1">
        <p className="text-xs uppercase tracking-wide text-muted">Output folder</p>
        <p className="truncate text-sm font-medium">{currentPath}</p>
      </div>
      <button type="button" className="btn-ghost" onClick={onChoose}>
        {accentLabel}
      </button>
    </div>
  );
}

export default OutputFolderChooser;
