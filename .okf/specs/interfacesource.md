---
type: Spec
title: InterfaceSource
description: Proveniência de contratos externos REST e SOAP para o IES
---

# InterfaceSource

Este tipo registra a fonte e o alcance de uma interface oficial publicada.

Campos do frontmatter:

- `protocol`: `rest-openapi` ou `soap-wsdl`.
- `contract_status`: `published-static` ou `per-installation-dynamic`.
- `runtime_availability`: `unknown` até verificação autorizada na instalação.
  Não confundir com um estado global; o registro operacional precisa de
  instalação, identidade/sistema, unidade e operação.
- A consulta WSDL pode exigir credenciais, mesmo sendo um GET de documento.
- `source_commit`: SHA de referência somente quando a fonte é Git versionado.
- `document_version`: versão da especificação, quando publicada.

Um contrato publicado não confere permissões, não demonstra equivalência
com a CLI e não autoriza chamadas live sem credenciais e ambiente adequado.
O tipo não é autorização para criar automaticamente backends REST/SOAP.
