import type { Metadata } from "next";
import { Source_Code_Pro } from "next/font/google";
import Link from "next/link";

import { Divider } from "@/components/Sheet";
import "./globals.css";

const sourceCodePro = Source_Code_Pro({
  variable: "--font-source-code-pro",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Pokerbot Viewer",
  description: "Replay hands from Pokerbot tournament logs",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="en" className={`${sourceCodePro.variable} h-full antialiased`}>
      <body className="flex min-h-full flex-col">
        <div className="mx-auto flex w-full max-w-5xl flex-1 flex-col px-4 py-6">
          <header className="flex flex-wrap items-baseline justify-between gap-x-4">
            <Link href="/" className="uppercase hover:bg-warn hover:text-[#111]">
              ♠ POKERBOT VIEWER
            </Link>
            <span className="text-muted uppercase">Match logs · hand replays</span>
          </header>
          <Divider />
          <main className="flex flex-1 flex-col gap-6 py-4">{children}</main>
        </div>
      </body>
    </html>
  );
}
