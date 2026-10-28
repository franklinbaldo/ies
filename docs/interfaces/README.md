---
type: Guide
title: Contratos oficiais de integração SEI
description: Fontes versionadas de REST e referência SOAP sem presumir disponibilidade
---

# Interfaces de integração conhecidas

O IES mantém **referências de origem**, não cópias gigantes de contratos
externos: [REST WSSEI OpenAPI](rest-wssei.md),
[SOAP SEI WSDL](soap-sei.md), [índice REST por endpoint](rest-endpoints.md)
e [controle de acesso por instalação](acesso-por-instalacao.md).

## Diferença fundamental

- O OpenAPI da REST WSSEI é **arquivo estático publicado no Git** e pode
  ser fixado por commit e hash de conteúdo.
- O WSDL SOAP do SEI é **produzido/servido por instalação** e deve ser
  consultado na instalação autorizada. Um manual de versão não substitui
  o contrato dinâmico do serviço específico.

**Na prática, algumas instalações não oferecem REST funcional e o
WSDL SOAP pode exigir credenciais até para consulta.** O catálogo de
interfaces não autoriza probing anônimo nem descoberta por tentativa e erro.

O [modelo de instalação, órgão, unidade e versão](instalacoes-e-versoes.md)
separa ambiente técnico, estrutura administrativa e contexto de sessão.
A versão do SEI é metadado, **não um indicador automático de compatibilidade**.

O [modelo de instalação, órgão, unidade e versão](instalacoes-e-versoes.md)
separa ambiente técnico, estrutura administrativa e contexto de sessão.
A versão do SEI é metadado, **não um indicador automático de compatibilidade**.

A disponibilidade de módulos, o modelo de autenticação e as permissões
são externos à CLI. Contratos publicados não comprovam equivalência
de campos, coleções, paginação nem escopo da unidade ativa.

O [ADR de seleção](../arquitetura/0001-rotas-por-evidencia.md) e o
[experimento comparativo](../experimentos/comparar-acessos-sei.md)
definem quando REST ou SOAP poderia substituir uma operação Web.
Nenhum backend deve ser escolhido por mera existência de endpoint.
