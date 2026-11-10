---
type: SEIObservation
title: "UnB de treinamento: processo enviar --remover-anotacoes, PASS com saída 0"
description: "Ensaio de 10/11/2026 da #333: envio com --manter-aberto e --remover-anotacoes na primeira tentativa, saída 0 e releitura independente; processo sem anotação prévia"
---

# Processo enviar --remover-anotacoes — UnB de treinamento, 10/11/2026

**Issue:** [#333](https://github.com/franklinbaldo/ies/issues/333).
**Origem:** somente `https://sei.treinamento.unb.br/sei` (UnB SEI 4.0.9 de
**treinamento**), conta pública `aluno1`, unidade `Unidade1`; sessão em
pasta `0700` com arquivos `0600`, fora do repositório.
**Processo:** `23106.001860/2026-25`, Gerado da Unidade1.
**Execução:** binário release de `main@10927ab`, sem alteração de código;
rede direta do sandbox. **Autorização:** pedido do proprietário nesta sessão.

## Orçamento

Declarado **30**; usado **18** requisições contadas pela CLI. O
`--confirmar` foi executado uma única vez, com trava `O_EXCL`.

## Sequência

| # | Comando | Resultado | Req. |
| --- | --- | --- | --- |
| 1 | `ies --json login` | `sessao_existente_valida` | 1 |
| 2 | `processo enviar NUP --para Unidade2 --manter-aberto --remover-anotacoes` (prévia) | PASS: destino `Unidade2` (`110000847`), `depois_esperado: [Unidade1, Unidade2]`, `metodo_remocao: checkbox_no_envio`, `anotacao_antes: false` | 5 |
| 3 | `… --confirmar` | **PASS saída 0**, `confirmado`, `alterado`, `depois: [Unidade1, Unidade2]`, `anotacao_depois: false` | 8 |
| 4 | `ies --json processo NUP` | aberto em `Unidade1` (atribuído a `aluno1`) e `Unidade2` | 4 |

## Alcance

A tramitação com `--manter-aberto` e o envio do checkbox
`chkSinRemoverAnotacoes` estão comprovados por releitura independente. O
processo **não tinha anotação**: a remoção de uma anotação existente não
foi exercitada ao vivo (o IES não tem comando para criar anotação). Se a
releitura encontrar anotação depois do envio, a CLI sai com efeito
incerto (3), sem repetir.

## Resultado e resíduo

**PASS.** Pela decisão do proprietário de 10/11/2026, a escrita fica
aprovada para produção (`src/origem_de_treinamento.rs`).
Resíduo: o processo `23106.001860/2026-25` ficou aberto também na
`Unidade2`. Não foi desfeito, porque concluir lá exigiria trocar a unidade
da sessão compartilhada.
