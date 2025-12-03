import React, { useEffect, useRef } from 'react';

const StarField: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animationFrameId: number;
    let stars: Array<{x: number, y: number, z: number, sz: number}> = [];
    const numStars = 400;

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };

    const initStars = () => {
      stars = [];
      for(let i=0; i<numStars; i++) {
        stars.push({
          x: Math.random() * canvas.width - canvas.width/2,
          y: Math.random() * canvas.height - canvas.height/2,
          z: Math.random() * canvas.width,
          sz: Math.random() * 2
        });
      }
    };

    const draw = () => {
      // Dark background with slight trail
      ctx.fillStyle = 'rgba(5, 5, 8, 0.2)';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      const cx = canvas.width / 2;
      const cy = canvas.height / 2;

      stars.forEach(star => {
        star.z -= 0.5;
        if (star.z <= 0) {
          star.z = canvas.width;
          star.x = Math.random() * canvas.width - canvas.width/2;
          star.y = Math.random() * canvas.height - canvas.height/2;
        }

        const x = (star.x / star.z) * canvas.width + cx;
        const y = (star.y / star.z) * canvas.height + cy;
        const sizeRaw = (1 - star.z / canvas.width) * 3 * star.sz;
        const size = Number.isFinite(sizeRaw) ? Math.max(0, sizeRaw) : 0;
        const alphaRaw = (1 - star.z / canvas.width);
        const alpha = Number.isFinite(alphaRaw) ? Math.min(1, Math.max(0, alphaRaw)) : 0;

        if (size > 0 && x >= 0 && x < canvas.width && y >= 0 && y < canvas.height) {
          ctx.beginPath();
          ctx.fillStyle = `rgba(212, 175, 55, ${alpha})`;
          ctx.arc(x, y, size, 0, Math.PI * 2);
          ctx.fill();
        }
      });

      animationFrameId = requestAnimationFrame(draw);
    };

    window.addEventListener('resize', resize);
    resize();
    initStars();
    draw();

    return () => {
      window.removeEventListener('resize', resize);
      cancelAnimationFrame(animationFrameId);
    };
  }, []);

  return (
    <canvas 
      ref={canvasRef} 
      className="absolute inset-0 z-0 pointer-events-none opacity-60"
    />
  );
};

export default StarField;
