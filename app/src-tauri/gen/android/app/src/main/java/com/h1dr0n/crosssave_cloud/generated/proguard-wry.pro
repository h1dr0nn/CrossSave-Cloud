# THIS FILE IS AUTO-GENERATED. DO NOT MODIFY!!

# Copyright 2020-2023 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

-keep class com.h1dr0n.crosssave_cloud.* {
  native <methods>;
}

-keep class com.h1dr0n.crosssave_cloud.WryActivity {
  public <init>(...);

  void setWebView(com.h1dr0n.crosssave_cloud.RustWebView);
  java.lang.Class getAppClass(...);
  java.lang.String getVersion();
}

-keep class com.h1dr0n.crosssave_cloud.Ipc {
  public <init>(...);

  @android.webkit.JavascriptInterface public <methods>;
}

-keep class com.h1dr0n.crosssave_cloud.RustWebView {
  public <init>(...);

  void loadUrlMainThread(...);
  void loadHTMLMainThread(...);
  void evalScript(...);
}

-keep class com.h1dr0n.crosssave_cloud.RustWebChromeClient,com.h1dr0n.crosssave_cloud.RustWebViewClient {
  public <init>(...);
}
