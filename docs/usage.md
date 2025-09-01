
# Usage

This document explains how to use the Rust Forward Proxy to intercept requests from a browser or at the DNS level.

## Browser Configuration

Most modern web browsers allow you to configure a proxy server. Here's how to do it in some popular browsers:

### Mozilla Firefox

1.  Go to **Preferences** > **General** > **Network Settings** > **Settings**.
2.  Select **Manual proxy configuration**.
3.  Enter the IP address and port of your proxy server in the **HTTP Proxy** field.
4.  Click **OK**.

### Google Chrome

Chrome uses the system's proxy settings. You can configure the proxy settings for your entire system, and Chrome will use them automatically.

#### Windows

1.  Go to **Settings** > **Network & Internet** > **Proxy**.
2.  Under **Manual proxy setup**, turn on **Use a proxy server**.
3.  Enter the IP address and port of your proxy server.
4.  Click **Save**.

#### macOS

1.  Go to **System Preferences** > **Network**.
2.  Select your active network connection (e.g., Wi-Fi) and click **Advanced**.
3.  Go to the **Proxies** tab.
4.  Check the box for **Web Proxy (HTTP)**.
5.  Enter the IP address and port of your proxy server.
6.  Click **OK** and then **Apply**.

## DNS-Level Interception

To intercept requests at the DNS level, you can use a DNS server that allows you to redirect traffic to your proxy server. This is a more advanced setup and requires a DNS server that supports this feature.

One way to achieve this is to use a local DNS server like `dnsmasq` to resolve specific domains to your proxy's IP address. For example, you could configure `dnsmasq` to resolve `*.example.com` to `127.0.0.1`, where your proxy is running. This would cause all requests to `*.example.com` to be sent to your proxy.

Here's an example of how you might configure `dnsmasq`:

```
address=/example.com/127.0.0.1
```

This would cause all requests to `example.com` and its subdomains to be resolved to `127.0.0.1`.

**Note:** DNS-level interception can be complex and may have unintended side effects. It's important to understand how it works before implementing it.
