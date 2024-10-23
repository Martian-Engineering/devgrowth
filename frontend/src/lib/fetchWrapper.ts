import { signOut } from "next-auth/react";
import { redirect } from "next/navigation";
import { logError } from "./logger";

export async function fetchWrapper(url: string, options: RequestInit = {}) {
  let response;
  try {
    response = await fetch(url, options);
    if (!response.ok) {
      const errorData = await response.text();
      throw new Error(`HTTP error! status: ${response.status} ${errorData}`);
    }
  } catch (error) {
    logError("Fetch error!", error);
    throw error;
  }

  // NOTE: there's something weird going on with the redirect logic here
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
