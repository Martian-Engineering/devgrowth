import { NextResponse } from "next/server";
import { cookies } from "next/headers";

export async function POST() {
  const apiUrl = process.env.NEXT_PUBLIC_API_URL;
  const targetUrl = `${apiUrl}/api/auth/logout`;

  const token = cookies().get("auth_token")?.value;
  const response = await fetch(targetUrl, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    credentials: "include",
  });

  const responseHeaders = new Headers(response.headers);
  responseHeaders.set(
    "Set-Cookie",
    "auth_token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; HttpOnly; Secure; SameSite=Strict",
  );

  // Also clear the next-auth session cookie
  responseHeaders.append(
    "Set-Cookie",
    "next-auth.session-token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; HttpOnly; Secure; SameSite=Lax",
  );

  return new NextResponse(null, {
    status: response.status,
    headers: responseHeaders,
  });
}
