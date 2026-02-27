import { useMemo, useState, useCallback, useEffect } from 'react'
import type { GeneratedFile } from '../lib/wasm'

interface Props {
  files: GeneratedFile[]
  activeFile: string
  onSelect: (path: string) => void
  maxHeight?: number
}

interface TreeNode {
  name: string
  fullPath: string
  path?: string // leaf nodes have a path
  children: TreeNode[]
}

function buildTree(files: GeneratedFile[]): TreeNode[] {
  const root: TreeNode[] = []

  for (const file of files) {
    const parts = file.path.split('/')
    let current = root
    let accumulated = ''

    for (let i = 0; i < parts.length; i++) {
      const name = parts[i]
      const isLeaf = i === parts.length - 1
      accumulated = accumulated ? `${accumulated}/${name}` : name

      let node = current.find((n) => n.name === name)
      if (!node) {
        node = {
          name,
          fullPath: accumulated,
          path: isLeaf ? file.path : undefined,
          children: [],
        }
        current.push(node)
      }
      current = node.children
    }
  }

  return root
}

function collectFolderPaths(nodes: TreeNode[]): string[] {
  const paths: string[] = []
  for (const node of nodes) {
    if (!node.path && node.children.length > 0) {
      paths.push(node.fullPath)
      paths.push(...collectFolderPaths(node.children))
    }
  }
  return paths
}

function TreeItem({
  node,
  depth,
  activeFile,
  onSelect,
  expanded,
  onToggle,
}: {
  node: TreeNode
  depth: number
  activeFile: string
  onSelect: (path: string) => void
  expanded: Set<string>
  onToggle: (path: string) => void
}) {
  const isFile = !!node.path
  const isActive = node.path === activeFile
  const isExpanded = expanded.has(node.fullPath)

  if (isFile) {
    return (
      <button
        onClick={() => onSelect(node.path!)}
        className={`flex items-center gap-1.5 w-full text-left px-2 py-1 text-xs rounded transition-colors ${
          isActive
            ? 'bg-white/10 text-white'
            : 'text-white/50 hover:text-white/80 hover:bg-white/5'
        }`}
        style={{ paddingLeft: `${depth * 12 + 8}px` }}
      >
        <span className="text-[10px] opacity-40">
          {node.name.endsWith('.ak') ? 'ak' : node.name.endsWith('.toml') ? 'tm' : 'ts'}
        </span>
        <span className="truncate">{node.name}</span>
      </button>
    )
  }

  return (
    <div>
      <button
        onClick={() => onToggle(node.fullPath)}
        className="flex items-center gap-1.5 w-full text-left px-2 py-1 text-xs text-white/30 hover:text-white/50 transition-colors"
        style={{ paddingLeft: `${depth * 12 + 8}px` }}
      >
        <svg
          width="10"
          height="10"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.5"
          className={`transition-transform duration-150 ${isExpanded ? 'rotate-90' : 'rotate-0'}`}
        >
          <polyline points="9 6 15 12 9 18" />
        </svg>
        <span>{node.name}</span>
      </button>
      <div
        className={`grid transition-[grid-template-rows] duration-150 ${isExpanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'}`}
      >
        <div className="overflow-hidden">
          {node.children.map((child) => (
            <TreeItem
              key={child.name}
              node={child}
              depth={depth + 1}
              activeFile={activeFile}
              onSelect={onSelect}
              expanded={expanded}
              onToggle={onToggle}
            />
          ))}
        </div>
      </div>
    </div>
  )
}

export default function FileTree({ files, activeFile, onSelect, maxHeight }: Props) {
  const tree = useMemo(() => buildTree(files), [files])
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set(collectFolderPaths(tree)))

  // Re-expand all when files change (new template selected)
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setExpanded(new Set(collectFolderPaths(tree)))
  }, [tree])

  const toggle = useCallback((path: string) => {
    setExpanded((prev) => {
      const next = new Set(prev)
      if (next.has(path)) {
        next.delete(path)
      } else {
        next.add(path)
      }
      return next
    })
  }, [])

  if (files.length === 0) {
    return (
      <div className="px-4 py-3 text-xs text-white/30">
        No files generated yet
      </div>
    )
  }

  return (
    <div className="py-2 overflow-y-auto" style={maxHeight ? { maxHeight } : { maxHeight: 192 }}>
      {tree.map((node) => (
        <TreeItem
          key={node.name}
          node={node}
          depth={0}
          activeFile={activeFile}
          onSelect={onSelect}
          expanded={expanded}
          onToggle={toggle}
        />
      ))}
    </div>
  )
}
