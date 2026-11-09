"use client";

import { useEffect, useRef, useState } from "react";

import { seatAngle } from "@/lib/seats";

/** Per-hand data for the grid, indexed by hand number − 1. Seat sets are bitmasks. */
export interface GridData {
  stillIn: number[];
  winners: number[];
  boardCards: number[];
}

/** The featured hand's current step, so its cell moves with the replay. */
export interface LiveCell {
  stillIn: number;
  winners: number;
  boardCards: number;
}

interface Props {
  totalHands: number;
  featured: number;
  tableSize: number;
  data: GridData;
  live: LiveCell;
  onPick: (hand: number) => void;
  onShownChange: (shown: number) => void;
}

const CELL_W = 34;
const CELL_H = 26;
const ROWS = 7;

/**
 * Rows of tiny tables, one per hand. Hands before the featured one are drawn
 * as played (who was still in, winners, board); later hands are faint, not
 * played yet. The featured hand sits inverted at a fixed spot.
 */
export function TableGrid({ totalHands, featured, tableSize, data, live, onPick, onShownChange }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [width, setWidth] = useState(0);
  const [scheme, setScheme] = useState(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const observer = new ResizeObserver(([entry]) => setWidth(entry.contentRect.width));
    observer.observe(canvas);
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const recolor = () => setScheme((n) => n + 1);
    media.addEventListener("change", recolor);
    return () => {
      observer.disconnect();
      media.removeEventListener("change", recolor);
    };
  }, []);

  const cols = Math.max(1, Math.floor(width / CELL_W));
  const offsetX = (width - cols * CELL_W) / 2;
  const featIndex = Math.floor(ROWS / 2) * cols + Math.floor(cols * 0.62);
  const firstHand = featured - featIndex;

  useEffect(() => {
    const shown = Math.min(totalHands, firstHand + cols * ROWS - 1) - Math.max(1, firstHand) + 1;
    onShownChange(Math.max(0, shown));
  }, [cols, firstHand, totalHands, onShownChange]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || width === 0) return;
    const ratio = window.devicePixelRatio || 1;
    const height = ROWS * CELL_H;
    canvas.width = Math.round(width * ratio);
    canvas.height = Math.round(height * ratio);
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
    ctx.clearRect(0, 0, width, height);

    const css = getComputedStyle(canvas);
    const ink = css.getPropertyValue("--ink").trim() || "#111";
    const paper = css.getPropertyValue("--paper").trim() || "#fff";
    const faint = css.getPropertyValue("--faint").trim() || "#ccc";

    for (let i = 0; i < cols * ROWS; i++) {
      const hand = firstHand + i;
      if (hand < 1 || hand > totalHands) continue;
      const x = offsetX + (i % cols) * CELL_W + CELL_W / 2;
      const y = Math.floor(i / cols) * CELL_H + CELL_H / 2;
      const isFeatured = hand === featured;
      const played = hand < featured;

      let color = played ? ink : faint;
      if (isFeatured) {
        ctx.fillStyle = ink;
        ctx.fillRect(x - CELL_W / 2 + 2, y - CELL_H / 2 + 2, CELL_W - 4, CELL_H - 4);
        color = paper;
      }

      const rx = CELL_W * 0.3;
      const ry = CELL_H * 0.27;
      ctx.strokeStyle = color;
      ctx.fillStyle = color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.ellipse(x, y, rx, ry, 0, 0, Math.PI * 2);
      ctx.stroke();
      if (!played && !isFeatured) continue;

      const cell = isFeatured
        ? live
        : { stillIn: data.stillIn[hand - 1], winners: data.winners[hand - 1], boardCards: data.boardCards[hand - 1] };
      for (let seat = 0; seat < tableSize; seat++) {
        const won = (cell.winners >> seat) & 1;
        const active = (cell.stillIn >> seat) & 1;
        const radius = won ? 3 : active ? 1.7 : 0;
        if (!radius) continue;
        const angle = seatAngle(seat, tableSize);
        ctx.beginPath();
        ctx.arc(x + Math.cos(angle) * rx, y + Math.sin(angle) * ry, radius, 0, Math.PI * 2);
        ctx.fill();
      }
      if (cell.boardCards) {
        const boardWidth = cell.boardCards * 3 - 1;
        for (let b = 0; b < cell.boardCards; b++) {
          ctx.fillRect(Math.round(x - boardWidth / 2) + b * 3, y - 2, 2, 4);
        }
      }
    }
  }, [width, cols, offsetX, firstHand, featured, totalHands, tableSize, data, live, scheme]);

  const pick = (event: React.MouseEvent<HTMLCanvasElement>) => {
    const rect = event.currentTarget.getBoundingClientRect();
    const col = Math.floor((event.clientX - rect.left - offsetX) / CELL_W);
    const row = Math.floor((event.clientY - rect.top) / CELL_H);
    if (col < 0 || col >= cols || row < 0 || row >= ROWS) return;
    const hand = firstHand + row * cols + col;
    if (hand >= 1 && hand <= totalHands) onPick(hand);
  };

  return (
    <figure className="my-2">
      <canvas
        ref={canvasRef}
        onClick={pick}
        role="img"
        aria-label={`Grid of tiny tables, one per hand, with hand ${featured} featured`}
        className="block w-full cursor-pointer"
        style={{ height: ROWS * CELL_H }}
      />
    </figure>
  );
}
