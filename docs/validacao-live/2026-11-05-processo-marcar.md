---
type: SEIObservation
title: Estado da validação de processo marcar no treinamento em 05/11/2026
description: Validação live e replay ainda não executados para a escrita do marcador da issue 255
---

# Validação live de `ies processo marcar` — 05/11/2026

| Etapa | Estado | Requisições |
| --- | --- | --- |
| Leitura estrutural de marcadores | PASS (evidência preexistente de 04/11, UnB) | consultar relatório original |
| Prévia deste branch na UnB | **NOT_RUN** | 0 |
| `--confirmar` de atribuição | **NOT_RUN** | 0 |
| `--confirmar` de retirada | **NOT_RUN** | 0 |
| Verificação posterior por leitura | **NOT_RUN** | 0 |
| Replay `marcador-escritas-*` | **NOT_RUN** | 0 |

O executor não dispõe de Rust e não resolve `github.com` nem
`static.rust-lang.org`; o acesso à instalação de treinamento não foi
estabelecido. Nenhuma escrita foi realizada, portanto **resíduo no SEI:
nenhum**. Não atribuir PASS por testes sintéticos ou por CI.

Antes de habilitar escrita: observar o formulário da unidade e seu POST,
rodar prévia live, enviar **uma vez** com `--confirmar` só em treinamento,
reler os marcadores e registrar efeito, requisições e resíduos.
