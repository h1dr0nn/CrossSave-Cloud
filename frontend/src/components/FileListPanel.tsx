export type FileItem = {
  id: string;
  name: string;
  size: string;
  status?: 'synced' | 'pending' | 'changed';
};

type Props = {
  files: FileItem[];
  onSelect?: (id: string) => void;
};

const statusStyles: Record<NonNullable<FileItem['status']>, string> = {
  synced: 'bg-green-500/80',
  pending: 'bg-yellow-400/80',
  changed: 'bg-orange-500/80'
};

function FileListPanel({ files, onSelect }: Props) {
  return (
    <div className="panel space-y-3">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-xs uppercase tracking-wide text-muted">Tracked Saves</p>
          <h2 className="text-lg font-semibold">Local Files</h2>
        </div>
        <button type="button" className="btn-ghost text-sm">
          Refresh
        </button>
      </div>
      <div className="rounded-soft border border-border/60 bg-white/50 shadow-inner-glow backdrop-blur-glass dark:bg-white/5">
        <ul className="divide-y divide-border/60">
          {files.map((file) => (
            <li
              key={file.id}
              className="group flex items-center justify-between px-4 py-3 transition duration-ui hover:bg-white/50 hover:shadow-soft-glow dark:hover:bg-white/10"
            >
              <div>
                <p className="font-medium">{file.name}</p>
                <p className="text-xs text-muted">{file.size}</p>
              </div>
              <div className="flex items-center gap-3">
                {file.status && (
                  <span
                    className={`h-2 w-2 rounded-full ${statusStyles[file.status]} shadow-soft-glow`}
                    aria-label={file.status}
                  />
                )}
                <button
                  type="button"
                  className="btn-ghost opacity-0 group-hover:opacity-100"
                  onClick={() => onSelect?.(file.id)}
                >
                  Select
                </button>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

export default FileListPanel;
