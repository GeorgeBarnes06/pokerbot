// Shared pieces of the printed-sheet layout.

import Link from "next/link";

/** A full-width rule of `=` characters, clipped to the container. */
export function Divider() {
  return (
    <div aria-hidden className="overflow-hidden whitespace-nowrap text-faint select-none">
      {"=".repeat(400)}
    </div>
  );
}

/** A numbered section: "EXHIBIT A: TITLE", with an optional note on the right. */
export function Exhibit({
  label,
  title,
  aside,
  children,
}: {
  label: string;
  title: string;
  aside?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section className="flex flex-col gap-3">
      <div className="flex flex-wrap items-baseline justify-between gap-x-4">
        <h2 className="font-normal uppercase">
          EXHIBIT {label}: {title}
        </h2>
        {aside && <div className="text-muted uppercase">{aside}</div>}
      </div>
      {children}
    </section>
  );
}

/** An inline link styled as `[TEXT]`. */
export function BracketLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <Link href={href} className="uppercase hover:bg-warn hover:text-[#111]">
      [{children}]
    </Link>
  );
}

/** A label/value pair for stat lines, e.g. "HANDS 199". */
export function Stat({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <span className="whitespace-nowrap">
      <span className="text-muted">{label}</span> {value}
    </span>
  );
}
