---
type: OperationEvidence
title: Validação da prévia de Acompanhamento Especial em 5 de novembro
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
  de inclusão e retirada; resultado do CI depende do head publicado.

## Limites

O comando `--confirmar` retorna recusa tipada antes de carregar
qualquer sessão; não há escrita nem autorização implícita para usar
produção. A próxima fatia deve implementar POST único derivado da
tela, conferir efeito por leitura, avaliar efeito incerto com saída 3,
medir o orçamento exato com mocks e depois validar ao vivo em
**treinamento** na ordem prévia → único confirmar → leitura.
