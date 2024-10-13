import { NextResponse } from "next/server";
import { cookies } from "next/headers";
import { getToken } from "next-auth/jwt";
import { logError } from "@/lib/logger";
import jwt from "jsonwebtoken";

const API_METHODS = ["GET", "POST", "PUT", "DELETE"];

export async function handler(request) {
  const apiUrl = process.env.NEXT_PUBLIC_API_URL;
  const url = new URL(request.url);

  // Don't proxy Next-auth routes
  if (url.pathname.startsWith("/api/auth")) {
    return NextResponse.next();
  }

  const targetUrl = `${apiUrl}${url.pathname}${url.search}`;

  // Only allow specified HTTP methods
  if (!API_METHODS.includes(request.method)) {
    return NextResponse.json({ error: "Method Not Allowed" }, { status: 405 });
  }

  try {
    // Get the session token
    let token = cookies().get("auth_token")?.value;
    if (!token) {
      const sessionToken = await getToken({ req: request });
      if (!sessionToken) {
        return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
      }
      token = jwt.sign(
        {
          name: sessionToken.name,
          email: sessionToken.email,
          exp: sessionToken.exp,
          iat: sessionToken.iat,
          id: sessionToken.id,
          access_token: sessionToken.accessToken,
        },
        process.env.NEXTAUTH_SECRET,
      );
    }

    // Forward all cookies
    const cookieStore = cookies();
    const allCookies = cookieStore.getAll();
    const cookieHeader = allCookies
      .map((cookie) => `${cookie.name}=${cookie.value}`)
      .join("; ");

    const headers = new Headers(request.headers);
    headers.set("Cookie", cookieHeader);
    // headers.set("Content-Type", "application/json");
    headers.set("Authorization", `Bearer ${token}`);

    const clonedRequest = request.clone();

    const fetchOptions = {
      method: clonedRequest.method,
      headers: headers,
      credentials: "include",
    };

    // // Add the body for POST, PUT, and PATCH requests
    if (["POST", "PUT", "PATCH"].includes(clonedRequest.method)) {
      // Simply forward the body as-is
      fetchOptions.body = clonedRequest.body;
      fetchOptions.duplex = "half";
    }

    const response = await fetch(targetUrl, fetchOptions);

    if (!response.ok) {
      console.log(response);
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    // Check for a new token in the response headers
    const newToken = response.headers.get("Authorization");
    if (newToken && newToken.startsWith("Bearer ")) {
      await storeNewToken(newToken.slice(7));
    }

    // Forward the Set-Cookie header if present
    const responseHeaders = new Headers();
    if (response.headers.has("Set-Cookie")) {
      responseHeaders.set("Set-Cookie", response.headers.get("Set-Cookie"));
    }

    if (response.status === 204) {
      return new NextResponse(null, {
        status: 204,
        headers: responseHeaders,
      });
    }

    let data = null;
    if (response.body) data = await response.json();

    return NextResponse.json(data, {
      status: response.status,
      headers: responseHeaders,
    });
  } catch (error) {
    logError("API call failed:", error);
    return NextResponse.json(
      { error: "Internal Server Error" },
      { status: 500 },
    );
  }
}

async function storeNewToken(token) {
  // You'll need to implement this based on your storage mechanism
  // For example, if you're using cookies:
  cookies().set("auth_token", token, {
    httpOnly: true,
    secure: true,
    sameSite: "strict",
  });
}

export const GET = handler;
export const POST = handler;
export const PUT = handler;
export const DELETE = handler;
