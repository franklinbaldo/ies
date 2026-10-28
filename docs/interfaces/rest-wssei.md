---
type: InterfaceSource
title: REST oficial WSSEI OpenAPI
protocol: rest-openapi
contract_status: published-static
runtime_availability: unknown
document_version: 3.0.0
source_commit: c6bfce09bd181a5f245e9dbb596d5916dba69727
---

# Módulo REST WSSEI — OpenAPI oficial

O [repositório PEN mod-wssei](https://github.com/pengovbr/mod-wssei)
contém o contrato da API do módulo adicional de integração REST.

- [OpenAPI JSON fixado por commit](https://github.com/pengovbr/mod-wssei/blob/c6bfce09bd181a5f245e9dbb596d5916dba69727/swagger/wssei.json)
- [OpenAPI YAML fixado por commit](https://github.com/pengovbr/mod-wssei/blob/c6bfce09bd181a5f245e9dbb596d5916dba69727/swagger/wssei.yaml)
- SHA-256 do JSON: `f2186a89d3f2bccd160d925d44be1c0e74926869db34a16f1b34015125865b13`.
- Inventário dos [127 endpoints](rest-endpoints.md), 127 operações HTTP
  e 34 schemas declarados no OpenAPI 3.0.0.

Para recuperar os bytes exatos do arquivo, sem credenciais:

```sh
curl -fsSL -o /tmp/wssei-openapi.json \
  https://raw.githubusercontent.com/pengovbr/mod-wssei/c6bfce09bd181a5f245e9dbb596d5916dba69727/swagger/wssei.json
echo 'f2186a89d3f2bccd160d925d44be1c0e74926869db34a16f1b34015125865b13  /tmp/wssei-openapi.json' | sha256sum -c -
```

## Autenticação e escopo observados no contrato

O esquema chama-se `BearerAuth`, mas sua declaração OpenAPI é
`type: apiKey`, `in: header`, `name: token`. Não presumir
automaticamente o prefixo `Bearer`.

O `GET /processo/listar` recebe `token` no cabeçalho,
`unidade` num cabeçalho opcional e parâmetros de paginação
`limit` e `start`, dentre outros. A API declara resposta JSON com
`data` e `total`, o que **não prova** correspondência com as duas
tabelas Recebidos/Gerados da caixa HTML do SEI.

Também consta `GET /usuario/unidades`, mas isso não comprova
acesso sem módulo instalado, token ou escopo autorizado.

Os `servers` de exemplo no OpenAPI usam URLs HTTP de ambientes PEN.
Esses exemplos **não são permissão de acesso**, nem orientação para
enviar tokens em HTTP. Um cliente real deve exigir HTTPS confiável
e validar a instância de destino.

## Decisão de engenharia

Uma instalação SEI **não é obrigada a ter o módulo WSSEI**.
Confirme versão, presença, autenticação, filtros e equivalência antes
de usar REST como implementação. Não coletar dados de produção
para provar disponibilidade. Não executar POST para mera descoberta.

Veja o [catálogo SOAP](soap-sei.md) e
[o experimento de comparação](../experimentos/comparar-acessos-sei.md).
