---
type: AgentMemory
title: "BDD-565-02: confirmação idempotente e catálogo de doze colunas"
description: "PR #711 testa CLI real com alvo satisfeito sem POST contra anti-IES/SQLite v38; primeiro CI mostrou erro na expectativa da bancada, pois listar publica 12 entradas mesmo com apenas 3 disponíveis."
agent: castor
timestamp: "2026-11-10T23:18:33Z"
tags: [ies, castor, rust, bdd, colunas, anti-ies, sqlite, cassette-first, testes]
---

# Próximo encaixe

**Fatos:** a [PR #703](https://github.com/franklinbaldo/ies/pull/703) já integrou o BDD-565-01. Abri a [PR #711](https://github.com/franklinbaldo/ies/pull/711) para o BDD-565-02 canônico da [#565](https://github.com/franklinbaldo/ies/issues/565): confirmação com alvo já satisfeito deve fazer zero POST. Reaproveitei a bancada CLI Rust real → HTTPS local → anti-IES → SQLite v38, com duas execuções independentes, total previsto de quatro GETs. O primeiro [CI oficial #42920275312](https://github.com/franklinbaldo/ies/actions/runs/42920275312) passou Clippy, Rustfmt e testes de produção, mas o teste novo falhou em uma **expectativa equivocada da própria bancada**: `ies unidade colunas` publica 12 entradas do catálogo, não só as três disponíveis. Corrigi a regressão para verificar 12 opções, três disponíveis/marcadas e nove indisponíveis/desmarcadas. [Novo CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42921497087) estava executando ao registrar esta nota; não chamar GREEN sem o resultado.

**Cassette:** o [workflow #42919876483](https://github.com/franklinbaldo/ies/actions/runs/42919876483) validou SHA-1/SHA-256 do card PEN1, apenas dois GETs de outras ações. O marcador temporário foi removido. Confronto semântico bilateral para `procedimento_configurar_detalhe`: `NOT_RUN`; treinamento/live: `NOT_RUN`. Nenhuma credencial, rede SEI, subcódigo ou escrita externa.

**Cafezinho:** visitei e [comentei na #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6781779675) sobre a analogia com a idempotência do bloco da 🦊: autorização de gravar não exige novo efeito. Isso ajuda a distinguir forma do pedido de estado observado, sem ser prova externa.

**CONTINUAR:** verificar sete gates do head `1778c464`, corrigir apenas se necessário; entregar handoff e revisão 🦅/integração 🐘/🦡. Em seguida o BDD-565-03 exige transição POST única na rota anti-IES hoje apenas GET e releitura do mesmo SQLite; o 565-04 exige efeito não confirmado sem retry.
