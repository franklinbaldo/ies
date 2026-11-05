---
type: OperationEvidence
title: Validação sintética da inclusão e pendência live de Acompanhamento Especial
description: Resultados observados e lacunas de validação para o handoff da issue 254
operation: processo.acompanhar
---

# Relatório da issue #254 — 05/11/2026

- **UnB / prévia Web:** NOT_RUN; **0 requisições**.
- **Segunda instalação / prévia:** NOT_RUN; **0 requisições**.
- **Escrita de acompanhamento em treinamento:** NOT_RUN; **0 POSTs**.
- **Resíduo criado ou alterado no SEI:** nenhum.
- **Replay do acervo:** NOT_RUN. Índices consultados, incluindo
  `processo-acompanhar-unb`, `processo-acompanhar-go`,
  `grupo-acompanhamento-ciclo-unb`.
- **RED local:** NOT_RUN (0 testes executados, número de falhas
  não observado). Sandbox sem `rustc` ou `cargo`; DNS não resolve
  `github.com`, impedindo buscar dependências e instalar Rust.
- **Testes sintéticos publicados:** seleção exata, ambiguidade, URL
  de outro processo, tabela ausente, IDs repetidos, reconciliação
  de inclusão e retirada, derivação exata do formulário, fluxo GET/POST
  com transporte injetado, orçamento 5 GETs na prévia e 7
  requisições na inclusão (3 abertura + gerenciar + cadastro + POST
  + releitura), falha de rede no POST, falha de releitura e ausência
  do efeito. CI deve ser conferido no head final antes de aprovação.

## Limites

A inclusão com `--confirmar` já passa por prévia e derivação
do formulário servido, envia um único POST, sem retry nem seguir
redirect, e faz releitura do gerenciador para conferir o efeito;
na ausência de prova de inclusão sai com código 3. A retirada
confirmada permanece bloqueada antes de carregar a sessão.
**Esta execução não usou credenciais nem enviou requisição real**,
portanto não comprova compatibilidade live. A validação autorizada
continua pendente exclusivamente em **treinamento**, na ordem
prévia → único confirmar → releitura, registrando o resíduo.
