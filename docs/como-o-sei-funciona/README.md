---
type: SEIObservation
title: Como o SEI funciona
description: Conhecimento empírico do SEI Web para implementação independente
---

# Como o SEI funciona

Este conjunto descreve **comportamentos do SEI**, não a arquitetura
ou a história de desenvolvimento do Todos. As principais medições
vieram de instalações de treinamento/homologação: UnB 4.0.9,
UFRJ 4.1.5, Goiás 5.0.3 e PEN (4.0–5.0), sobretudo em outubro de 2026.

Fatos observados valem no contexto medido. Quando houver uma inferência,
ela será identificada. Nunca generalize comportamento de uma instalação
sem comparação. [Cassettes](../cassettes.md) são a evidência original.

## Tópicos

- [Sessão e login](sessao-login.md)
- [URLs assinadas e infra_hash](infra-hash.md)
- [Rotas, links e árvore do processo](rotas-e-arvore.md)
- [Formulários e JavaScript](formularios.md)
- [Tabelas e paginação](tabelas-e-paginacao.md)
- [Estado e diferenças entre instalações](estado-e-instalacoes.md)
- [Efeitos e verificação](efeitos-e-verificacao.md)
- [Regressões recentes do Todos relevantes ao IES](alertas-do-todos.md)

Esses fatos ajudam a construir testes e compreender as capturas.
Não são licença para fabricar URLs, inferir permissões ou executar
escritas em telas ainda não mapeadas.
