---
# Front Matter (YAML).
## name - The name of the site. (max 64 characters)
name: Pain.001.001.03 Messages
## title - The title of the page. (max 64 characters)
title: Pain001
## subtitle - The subtitle of the page. (max 64 characters)
subtitle: A Python library that makes it easy to automate the creation of ISO20022-compliant payment files.
## author - The author of the site. (max 64 characters)
author: Pain001
## charset - The charset of the site. (default: utf-8)
charset: utf-8
## description - The description of the site. (max 160 characters)
description: A Python library that makes it easy to automate the creation of ISO20022-compliant payment files.
## keywords - The keywords of the site. (comma separated, max 10 keywords)
keywords: pain001, minimalist, professional, versatile, modern, clean, dynamic, elegant,user-friendly, responsive
## url - The url of the site.
permalink: /
## language - The language of the site. (default: en-GB)
language: en-GB
## layout - The layout of the site. (default: index) (see: templates)
layout: index
## icon - The icon of the site in SVG format.
icon: "https://kura.pro/pain001/images/icons/512x512.png"
## image - The main logo of the site in SVG format.
image: https://kura.pro/pain001/images/banners/banner-pain001.svg
## banner - The banner of the site.
banner: https://kura.pro/pain001/images/logos/pain001.svg
## theme_color - The theme color of the site.
theme_color: "rgb(33, 37, 41)"

# RSS - The RSS feed front matter (YAML).

## rss - The RSS feed link of the page.
atom_link: /rss.xml
## rss - The RSS last build date of the page.
last_build_date: 2023-03-23T00:00:00+00:00
## rss - The RSS pub date of the page.
pub_date: 2023-03-23T00:00:00+00:00
## rss - The RSS generator of the page.
generator: Pain001 🦀 (version 0.0.12)
## rss - The RSS item title of the page.
item_title: "RSS"
## rss - The RSS item link of the page.
item_link: /rss/
## rss - The RSS item guid of the page.
item_guid: 4ef3bf81-5515-45f0-9254-6d70caf0d75f
## rss - The RSS item description of the page.
item_description: RSS feed for the site
## rss - The RSS item pub date of the page.
item_pub_date: 2019-10-16T15:00:00+00:00

# MS Application - The MS Application front matter (YAML).

## msapplication - The MS Application config of the page.
msapplication_config: /browserconfig.xml
## msapplication_tap_highlight - The MS Application tap highlight of the page.
msapplication_tap_highlight: no
## msapplication - The MS Application tile color of the page.
msapplication_tile_color: "#7ce846"
## msapplication_tile_image - The MS Application tile image of the page.
msapplication_tile_image: https://kura.pro/pain001/images/logos/pain001.svg

# Open Graph - The Open Graph front matter (YAML).

## og - The Open Graph description of the page.
og_description: A Python library that makes it easy to automate the creation of ISO20022-compliant payment files.
## og - The Open Graph image of the page.
og_image: https://kura.pro/pain001/images/logos/pain001.svg
## og:image:alt - The Open Graph image alt of the page.
og_image_alt: Pain001 Logo
## og - The Open Graph locale of the page.
og_locale: en_GB
## og - The Open Graph site name of the page.
og_site_name: pain001.one
## og - The Open Graph title of the page.
og_title: Pain001
## og - The Open Graph type of the page.
og_type: website
## og - The Open Graph url of the page.
og_url: https://pain001.one

# Twitter Card - The Twitter Card front matter (YAML).

## twitter_card - The Twitter Card type of the page.
twitter_card: summary
## twitter_creator - The Twitter Card creator of the page.
twitter_creator: sebastienrousseau
## twitter_description - The Twitter Card description of the page.
twitter_description: A Python library that makes it easy to automate the creation of ISO20022-compliant payment files.
## twitter_image - The Twitter Card image of the page.
twitter_image: https://kura.pro/pain001/images/logos/pain001.svg
## twitter_image:alt - The Twitter Card image alt of the page.
twitter_image_alt: Pain001 Logo
## twitter_site - The Twitter Card site of the page.
twitter_site: sebastienrousseau
## twitter_title - The Twitter Card title of the page.
twitter_title: Pain001
## twitter_url - The Twitter Card url of the page.
twitter_url: https://pain001.one

# Google Analytics - The Google Analytics front matter (YAML).

## google_site_verification - The Google Analytics site verification of the page.
google_site_verification: 1234567890

# Bing Webmaster Tools - The Bing Webmaster Tools front matter (YAML).

## bing_site_verification - The Bing Webmaster Tools site verification of the page.
bing_site_verification: 1234567890

---

<!-- markdownlint-disable MD033 MD041 -->

<header class="bg-light bg-gradient container py-5 px-5">

<!-- markdownlint-enable MD033 MD041 -->

## Overview 📖

![Licenses](https://kura.pro/pain001/images/logos/pain001.svg "Licenses").class=\"float-end m-3 w-25\"
The `Pain001` Python package is a CLI tool that makes it easy to
automate the creation of ISO20022-compliant payment files directly from
a CSV file.

With `Pain001`, you can easily create payment transactions files in just
a few simple steps.

The library supports both **Single Euro Payments Area (SEPA)** and
**non-SEPA credit transfers**, making it versatile for use in different
countries and regions.

## ISO 20022 Payment Initiation Message Types 📨

The following **ISO 20022 Payment Initiation message types** are
currently supported:

* **pain.001.001.03** - Customer Credit Transfer Initiation

This message is used to transmit credit transfer instructions from the
originator (the party initiating the payment) to the originator's bank.
The message supports both bulk and single payment instructions, allowing
for the transmission of multiple payments in a batch or individual
payments separately. The pain.001.001.03 message format is part of the
ISO 20022 standard and is commonly used for SEPA Credit Transfers within
the Single Euro Payments Area. It includes relevant information such as
the originator's and beneficiary's details, payment amounts, payment
references, and other transaction-related information required for
processing the credit transfers.

* **pain.001.001.09** - Customer Credit Transfer Initiation

This message format is part of the ISO 20022 standard and is commonly
used for SEPA Credit Transfers within the Single Euro Payments Area. It
enables the transmission of credit transfer instructions from the
originator to the originator's bank. The message includes essential
information such as the originator's and beneficiary's details, payment
amounts, payment references, and other transaction-related information
required for processing the credit transfers.

More message types will be added in the future. Please refer to the
[supported messages section][supported-messages] section for more
details.

<!-- markdownlint-disable MD033 MD041 -->

<div class="d-grid gap-3 d-sm-flex justify-content-sm-center">
    <a class="btn btn-primary btn-lg px-4 me-sm-3" alt="Features for Pain001, a Pain001 starter template" href="#features">Features</a>
    <a class="btn btn-secondary btn-lg px-4 me-sm-3" alt="Learn more on Crates.io" href="https://crates.io/crates/ssg">Learn more on Crates.io</a>
</div>

</header>

## Features ✨

* **Simplify file creation:** The library generates payment files in
  the desired format quickly and efficiently.
* **Ensure the highest quality and compliance:** The library
  guarantees that all created payment files follow the ISO 20022
  standards.
* **Enhance efficiency:** The Pain001 library automates the creation of
  Payment Initiation message files, freeing developers to focus on other
  aspects of their projects and simplifying the payment process for
  users.
* **Improve accuracy:** By providing precise data, the library reduces
  errors in payment file creation and processing.
* **Seamless integration:** As a Python package, the Pain001 library is
  compatible with various Python-based applications and easily
  integrates into any existing projects or workflows.
* **Cross-border compatibility:** The library supports both Single Euro
  Payments Area (SEPA) and non-SEPA credit transfers, making it
  versatile for use in different countries and regions.
* **Time-saving:** The automated file creation process reduces the time
  spent on manual data entry and file generation, increasing overall
  productivity.
* **Scalable solution:** The Pain001 library can handle varying volumes
  of payment files, making it suitable for businesses of different sizes
  and transaction volumes.
* **Customisable:** The library allows developers to customise the
  output, making it adaptable to specific business requirements and
  preferences.

## Overview

![Licenses](https://kura.pro/pain001/images/logos/pain001.svg "Licenses").class=\"float-start m-3 w-25\" Pain001 is a minimalist and modern [Pain001][0] starter template designed for professionals who value simplicity and elegance. With its clean and dynamic layout, Pain001 offers a versatile and user-friendly solution for those looking to showcase their work and services online. Built on a responsive framework, this template is ideal for professionals without coding or design skills.

Whether you're a freelance creative, a startup founder, or a small business owner. Pain001's ready-to-use website and responsive starter templates provide the perfect foundation for your online presence. With its minimalist design, Pain001 is the ultimate website starter template for modern and professional websites.

This page is an example for the Pain001 static website generator. You can use it as a template for your website or blog. It uses a markdown template for the content and a custom HTML theme for the layout.

<!-- markdownlint-disable MD033 MD041 -->

<div class="d-grid gap-3 d-sm-flex justify-content-sm-center">
    <a class="btn btn-primary btn-lg px-4 me-sm-3" alt="Features for Pain001, a Pain001 starter template" href="#features">Features</a>
    <a class="btn btn-secondary btn-lg px-4 me-sm-3" alt="Learn more on Crates.io" href="https://crates.io/crates/ssg">Learn more on Crates.io</a>
</div>

</header>

<!-- markdownlint-enable MD033 MD041 -->

## Features

![Licenses](https://kura.pro/pain001/images/logos/pain001.svg "Licenses").class=\"float-end m-3 w-25\" Get started with Pain001 using any of our examples or use parts of them for building your custom layouts and content.

This template Pain001 has the following features enabled:

* **Responsive Navigation Bar:** The responsive navigation bar provides users with an intuitive and easy-to-use interface for navigating the website. It aims to adapt to the size of the screen, making it accessible to users on both desktop and mobile devices.
* **Open Graph/Facebook Meta-Tags:** These meta tags allow you to control how your website appears when shared on Facebook and other social media platforms. By setting the title, description, and image, you can make sure that your website looks its best when shared online.
* **Accessibility Meta-Tags:** These meta tags are designed to make the website more accessible to users with disabilities. By setting Accessible Rich Internet Applications (ARIA) roles and attributes, full keyboard control, and no flashing hazard, you can make sure your website is accessible to everyone.
* **Content Security Policy:** This meta tag is used to specify the sources of content allowed to load on the page. It is designed to prevent cross-site scripting (XSS) attacks and other security vulnerabilities.
* **Apple Meta-Tags:** These meta tags improve websites for Apple devices, like iPhones, iPads, and Apple devices. You can change web app capabilities, status bar style, title, application name, and author to improve Apple devices' appearance.
* **Bootstrap CSS:** Bootstrap is a popular CSS framework that provides you with a set of pre-designed styles and components. By using Bootstrap, you can quickly and easily create a professional-looking website without having to write CSS from scratch.
* **Bootstrap JavaScript:** Bootstrap JavaScript is a set of pre-built scripts that provide you with responsive navigation menus and modal dialogues.
* **Schema.org Meta Tags:** These meta tags are used to provide structured data about the website's content. Setting the name, description, and image on a website helps search engines and others understand the content better.
* **Microsoft Meta Tags:** These meta tags are designed to optimise the website for Microsoft devices. You can set site verification, application configuration, tap highlight colour, tile colour, and tile image to look good on Windows devices.
* **Twitter Meta Tags:** These meta tags are designed to optimise the website for Twitter sharing. You can set the card type, creator, description, image, site, title, and URL to make their website look good on Twitter.

[0]: https://pain001.one/
