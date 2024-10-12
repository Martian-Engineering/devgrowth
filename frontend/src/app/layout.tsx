import "./globals.css";
import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { Provider } from "./Provider";
import { Navbar } from "@/components/Navbar";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "DevGrowth",
  description: "Analyze GitHub repository growth",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <Provider>
          <Navbar />
          <main className="max-w-7x1 mx-auto py-6 sm:px-6 lg:px-8">
            {children}
          </main>
        </Provider>
      </body>
    </html>
  );
}
