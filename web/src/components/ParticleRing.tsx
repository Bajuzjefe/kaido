import { useRef, useEffect } from 'react'

// Canvas-based scattered particles that follow the mouse
// Reference: 80 particles × 25 rows = 2000 tiny dots, size 2px, slow 3s mouse follow
// Canvas is pointer-events-none; mouse tracking via parent section
export default function ParticleRing() {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const parent = canvas.parentElement
    if (!parent) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    let mouseX = 0.5
    let mouseY = 0.5
    let targetX = 0.5
    let targetY = 0.5
    let animFrame: number
    let lastTime = 0

    // Match reference: particle-count=80, particle-rows=25
    const PARTICLE_COUNT = 80
    const ROWS = 25

    const seed = 200
    function seededRandom(i: number) {
      const x = Math.sin(seed + i * 9301 + 49297) * 49297
      return x - Math.floor(x)
    }

    interface Particle {
      angle: number
      radiusNorm: number
      alpha: number
      size: number
    }

    const particles: Particle[] = []
    for (let i = 0; i < PARTICLE_COUNT * ROWS; i++) {
      particles.push({
        angle: seededRandom(i * 4) * Math.PI * 2,
        radiusNorm: seededRandom(i * 4 + 1),
        // Low alpha for subtle dust effect on dark bg
        alpha: 0.04 + seededRandom(i * 4 + 3) * 0.18,
        // Tiny dots: 0.5-1.5px (reference uses particle-size=2)
        size: 0.5 + seededRandom(i * 4 + 2) * 1,
      })
    }

    function resize() {
      const dpr = window.devicePixelRatio || 1
      const rect = canvas!.getBoundingClientRect()
      canvas!.width = rect.width * dpr
      canvas!.height = rect.height * dpr
      ctx!.setTransform(dpr, 0, 0, dpr, 0, 0)
    }

    function draw(timestamp: number) {
      if (!ctx || !canvas) return

      const dt = Math.min((timestamp - lastTime) / 1000, 0.05)
      lastTime = timestamp

      const w = canvas.getBoundingClientRect().width
      const h = canvas.getBoundingClientRect().height

      // Slow follow with exponential decay
      const followSpeed = 1 - Math.exp(-dt * 0.8)
      mouseX += (targetX - mouseX) * followSpeed
      mouseY += (targetY - mouseY) * followSpeed

      const cx = mouseX * w
      const cy = mouseY * h

      // Reference: ring-radius=100, ring-thickness=600
      // Inner radius small, outer extends far to fill viewport
      const viewSize = Math.max(w, h)
      const minRadius = viewSize * 0.06
      const maxRadius = viewSize * 0.55

      const time = Date.now() * 0.001
      // Gentle pulse (reference: ring animation 6s alternate between 150-250)
      const pulse = Math.sin(time * 0.5) * viewSize * 0.02

      ctx.clearRect(0, 0, w, h)

      for (const p of particles) {
        const r = minRadius + p.radiusNorm * (maxRadius - minRadius) + pulse
        const angle = p.angle + time * 0.03
        const x = cx + Math.cos(angle) * r
        const y = cy + Math.sin(angle) * r

        if (x < -10 || x > w + 10 || y < -10 || y > h + 10) continue

        ctx.beginPath()
        ctx.arc(x, y, p.size, 0, Math.PI * 2)
        // Muted blue-gray, like "navy" on dark bg but very subtle
        ctx.fillStyle = `rgba(140, 150, 180, ${p.alpha})`
        ctx.fill()
      }

      animFrame = requestAnimationFrame(draw)
    }

    // Listen on parent section so buttons remain clickable
    function onPointerMove(e: PointerEvent) {
      const rect = canvas!.getBoundingClientRect()
      targetX = (e.clientX - rect.left) / rect.width
      targetY = (e.clientY - rect.top) / rect.height
    }

    function onPointerLeave() {
      targetX = 0.5
      targetY = 0.5
    }

    resize()
    window.addEventListener('resize', resize)
    parent.addEventListener('pointermove', onPointerMove as EventListener)
    parent.addEventListener('pointerleave', onPointerLeave as EventListener)
    animFrame = requestAnimationFrame(draw)

    return () => {
      cancelAnimationFrame(animFrame)
      window.removeEventListener('resize', resize)
      parent.removeEventListener('pointermove', onPointerMove as EventListener)
      parent.removeEventListener('pointerleave', onPointerLeave as EventListener)
    }
  }, [])

  return (
    <canvas
      ref={canvasRef}
      className="absolute inset-0 w-full h-full pointer-events-none"
    />
  )
}
