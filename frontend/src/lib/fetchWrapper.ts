import { signOut } from "next-auth/react";
import { redirect } from "next/navigation";

export async function fetchWrapper(url: string, options: RequestInit = {}) {
  const response = await fetch(url, options);

  if (response.status === 401) {
    // Clear the session
    await signOut({ redirect: false });
    await fetch("/api/auth/logout", { method: "POST" });

    // redirect to home
    // TODO: redirect to login page when I have one
    redirect("/");
  }

  return response;
}
