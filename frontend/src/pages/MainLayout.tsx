import { useMemo } from 'react';
import FileListPanel, { FileItem } from '../components/FileListPanel';
import FormatSelector from '../components/FormatSelector';
import OutputFolderChooser from '../components/OutputFolderChooser';
import ProgressIndicator from '../components/ProgressIndicator';
import ToastMessage from '../components/ToastMessage';
import { useTheme } from '../hooks/useTheme';

const sampleFiles: FileItem[] = [
  { id: '1', name: 'Zelda.sav', size: '512 KB', status: 'synced' },
  { id: '2', name: 'Metroid.srm', size: '1.2 MB', status: 'pending' },
  { id: '3', name: 'Chrono Trigger.sav', size: '768 KB', status: 'changed' }
];

const formats = ['zip', 'tar.gz', '7z'];

function MainLayout() {
  const theme = useTheme();

  const sidebarItems = useMemo(
    () => [
      { label: 'Library', active: true },
      { label: 'Recent' },
      { label: 'Downloads' },
      { label: 'Settings' }
    ],
    []
  );

  return (
    <div className="flex min-h-screen gap-4 p-6 text-foreground">
      <aside className="panel flex w-64 flex-col justify-between p-4">
        <div className="space-y-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-muted">CrossSave</p>
              <p className="text-lg font-semibold">Cloud Sync</p>
            </div>
            <button
              type="button"
              className="btn-ghost rounded-full px-3"
              aria-label="Toggle theme"
              onClick={theme.toggle}
            >
              {theme.mode === 'dark' ? '🌙' : '☀️'}
            </button>
          </div>
          <nav className="space-y-2">
            {sidebarItems.map((item) => (
              <button
                key={item.label}
                className={`flex w-full items-center justify-between rounded-soft px-3 py-2 text-sm font-medium transition duration-ui hover:bg-white/40 hover:shadow-soft-glow dark:hover:bg-white/10 ${
                  item.active ? 'bg-white/50 selection-ring dark:bg-white/5' : 'glass-panel'
                }`}
              >
                <span>{item.label}</span>
                {item.active && <span className="h-2 w-2 rounded-full bg-accent" aria-hidden />}
              </button>
            ))}
          </nav>
        </div>
        <div className="rounded-soft bg-white/30 p-3 text-xs text-muted shadow-inner-glow backdrop-blur-glass dark:bg-white/5">
          <p className="font-medium text-foreground">Local Storage</p>
          <p>Connected · Last synced 2h ago</p>
        </div>
      </aside>

      <main className="flex flex-1 flex-col gap-4">
        <header className="panel flex items-center justify-between">
          <div>
            <p className="text-xs uppercase tracking-wide text-muted">Session</p>
            <h1 className="text-2xl font-semibold">Manual Packaging</h1>
          </div>
          <div className="flex items-center gap-3">
            <FormatSelector formats={formats} value="zip" onChange={() => {}} />
            <OutputFolderChooser
              currentPath="~/CrossSave/Exports"
              onChoose={() => {}}
              accentLabel="Choose folder"
            />
          </div>
        </header>

        <section className="grid grid-cols-3 gap-4">
          <div className="col-span-2 space-y-4">
            <FileListPanel files={sampleFiles} onSelect={() => {}} />
            <div className="panel flex items-center justify-between">
              <ProgressIndicator label="Packaging" progress={62} eta="~45s" status="Encrypting" />
              <div className="flex items-center gap-3">
                <button type="button" className="btn-ghost">
                  Cancel
                </button>
                <button type="button" className="btn-accent">
                  Pause
                </button>
              </div>
            </div>
          </div>
          <div className="space-y-4">
            <ToastMessage title="Synced" tone="success" message="Save archive uploaded to cloud." />
            <ToastMessage title="Watcher" tone="info" message="Monitoring RetroArch saves." />
            <ToastMessage title="Conflict" tone="warning" message="Version mismatch detected." />
          </div>
        </section>
      </main>
    </div>
  );
}

export default MainLayout;
