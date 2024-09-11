---
title: "Bringing Segmented Stacks to Embedded Systems"

# Authors
# If you created a profile for a user (e.g. the default `admin` user), write the username (folder name) here 
# and it will be replaced with their full name and linked to their profile.
authors:
- Zhiyao Ma
- Lin Zhong

share: false

# Author notes (optional)
# author_notes:
# - "Equal contribution"
# - "Equal contribution"

date: "2023-02-22T00:00:00Z"
doi: "10.1145/3572864.3580344"

# Schedule page publish date (NOT publication's date).
publishDate: "2023-02-22T00:00:00Z"

# Publication type.
# Legend: 0 = Uncategorized; 1 = Conference paper; 2 = Journal article;
# 3 = Preprint / Working Paper; 4 = Report; 5 = Book; 6 = Book section;
# 7 = Thesis; 8 = Patent
publication_types: ["1"]

# Publication name and optional abbreviated publication name.
publication: In Proceedings of the 24th International Workshop on Mobile Computing Systems and Applications
publication_short: In *HotMobile 2023*

abstract: Microcontrollers are the heart of embedded systems. Due to cost and power constraints, they do not have memory management units (MMUs) or even memory protection units (MPUs). As a result, embedded software faces two related challenges both concerned with the stack. First, in a multi-tasking environment, physical memory used by the stack is usually statically allocated per task. Second, a stack overflow is difficult to detect for lower-end microcontrollers without an MPU. In this work, we argue that segmented stacks, a notion investigated and subsequently dismissed for systems with virtual memory, can solve both challenges for embedded software. We show that many problems with segmented stacks vanish on embedded systems and present novel solutions to the rest. Importantly, we show that segmented stacks, combined with Rust, can guarantee memory safety without MMU or MPU. Moreover, segmented stacks allow memory to be dynamically allocated to per-task stacks and can improve memory efficiency when combined with proper scheduling.

# Summary. An optional shortened abstract.
# summary: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Duis posuere tellus ac convallis placerat. Proin tincidunt magna sed ex sollicitudin condimentum.

tags: [embedded, operating-system]

# Display this page in the Featured widget?
featured: true

# Custom links (uncomment lines below)
# links:
# - name: Custom Link
#   url: http://example.org

url_pdf: ''
url_code: ''
url_dataset: ''
url_poster: ''
url_project: ''
url_slides: ''
url_source: ''
url_video: ''

# Featured image
# To use, add an image named `featured.jpg/png` to your page's folder. 
# image:
#   caption: 'Image credit: [**Unsplash**](https://unsplash.com/photos/pLCdAaMFLTE)'
#   focal_point: ""
#   preview_only: false

# Associated Projects (optional).
#   Associate this publication with one or more of your projects.
#   Simply enter your project's folder or file name without extension.
#   E.g. `internal-project` references `content/project/internal-project/index.md`.
#   Otherwise, set `projects: []`.
# projects:
# - example

# Slides (optional).
#   Associate this publication with Markdown slides.
#   Simply enter your slide deck's filename without extension.
#   E.g. `slides: "example"` references `content/slides/example/index.md`.
#   Otherwise, set `slides: ""`.
# slides: example
---

<!-- {{% callout note %}}
Click the *Cite* button above to demo the feature to enable visitors to import publication metadata into their reference management software.
{{% /callout %}}

{{% callout note %}}
Create your slides in Markdown - click the *Slides* button to check out the example.
{{% /callout %}}

Supplementary notes can be added here, including [code, math, and images](https://wowchemy.com/docs/writing-markdown-latex/). -->
