// Seat placement shared by the featured table and the grid of tiny tables.

/** Seat 1 sits lower left, then clockwise, evenly spaced (45° apart for 8 seats). */
export function seatAngle(seat: number, tableSize: number): number {
  return ((112.5 + (seat * 360) / tableSize) * Math.PI) / 180;
}

/** Seat centre as percentages of the table figure, on an ellipse of radii rx/ry. */
export function seatPosition(seat: number, tableSize: number, rx: number, ry: number) {
  const angle = seatAngle(seat, tableSize);
  return { left: 50 + rx * Math.cos(angle), top: 50 + ry * Math.sin(angle) };
}
