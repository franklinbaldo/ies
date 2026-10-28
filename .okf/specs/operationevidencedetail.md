---
type: Spec
title: OperationEvidenceDetail
description: Continuação de evidências por instalação quando o card excederia 20 KB
---

# OperationEvidenceDetail

Desdobramento de uma operação por instalação, criado somente se um
card completo excederia o teto de 20.000 bytes. As relações são links
Markdown para `Cassette`; o documento aponta para seu card pai.

- `operation`: operação do card pai.
- `installation`: instalação objeto desta parte.

A divisão é por responsabilidade, não minificação para contornar o gate.
