import type { GeneratedFile } from '../lib/wasm'

interface Props {
  files: GeneratedFile[]
  projectName: string
}

export default function DownloadButton({ files, projectName }: Props) {
  const handleDownload = async () => {
    if (files.length === 0) return

    const { default: JSZip } = await import('jszip')
    const { saveAs } = await import('file-saver')

    const zip = new JSZip()
    for (const file of files) {
      zip.file(file.path, file.content)
    }

    const blob = await zip.generateAsync({ type: 'blob' })
    saveAs(blob, `${projectName}.zip`)
  }

  return (
    <button
      onClick={handleDownload}
      disabled={files.length === 0}
      className="btn-primary w-full justify-center py-2.5 text-sm disabled:opacity-30 disabled:cursor-not-allowed"
    >
      Download ZIP ({files.length} files)
    </button>
  )
}
