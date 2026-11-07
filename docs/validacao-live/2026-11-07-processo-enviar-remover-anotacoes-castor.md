---
type: Guide
title: "Validação da remoção de anotações no envio — 07/11/2026"
description: "Fatia BDD #333 com confirmação pelo checkbox; apenas testes offline, live não executado"
---

# Processo enviar com remoção de anotações — 07/11/2026

Issue [#333](https://github.com/franklinbaldo/ies/issues/333);
PR Draft [#401](https://github.com/franklinbaldo/ies/pull/401).

## Evidência disponível

Os testes de transporte **sintético** exercitam
`ies processo enviar NUP --para SIGLA --remover-anotacoes --confirmar`
com o formulário de envio que oferece `chkSinRemoverAnotacoes` e a tela
de anotações identificada pelo processo. O fluxo esperado contém três GET
de abertura, um GET da tela de envio, um GET de anotação anterior, um POST
de **consulta** ao autocomplete, um único POST **mutador** de envio, um GET
da árvore após a escrita e um GET independente da anotação posterior:
**9 tentativas sintéticas**.

A leitura posterior vazia e a situação do processo confirmada permitem
sucesso no cenário sintético. Anotação ainda presente é alteração parcial;
falha de releitura é efeito incerto (saída 3), sempre sem retry. Quando o
checkbox não está presente, o fallback `limpar_anotacao` continua sem
confirmação: nenhuma escrita sai nesse cenário.

A prova RED anterior à implementação está no
[CI #42394002024](https://github.com/franklinbaldo/ies/actions/runs/42394002024):
um teste novo falhou, 2.001 passaram e 25 estavam ignorados. Essa
evidência é **offline**, não um ensaio autenticado do SEI.

## Live de treinamento

| Etapa | Estado | Requisições reais |
| --- | --- | --- |
| Autenticação e prévia autenticada | **NOT_RUN** | 0 |
| Confirmação única com checkbox | **NOT_RUN** | 0 |
| Releitura da árvore e anotação | **NOT_RUN** | 0 |
| Fallback `txaDescricao=""` | **NOT_RUN** — escrita ainda não implementada | 0 |

**Nenhuma instalação de produção ou treinamento foi acessada nesta
rodada**. Zero GETs e POSTs HTTP reais ao SEI, zero escritas,
zero retries e **nenhum resíduo novo**. Não declarar PASS live, remoção
real ou compatibilidade de outras instalações a partir das fixtures.

## Próximo ensaio

Em instalação exclusivamente de **TREINAMENTO**, identificar processo
descartável e unidade autorizada; registrar orçamento antes da execução;
fazer prévia autenticada, um único `--confirmar` apenas quando a opção e
a anotação forem legíveis, e duas releituras independentes do efeito.
Registrar resposta real, contagem, unidade origem/destino e qualquer
resíduo. Caso a escrita ou releitura seja inconclusiva, sair com 3
e **não repetir POST**.

Para instalações sem checkbox, o método alternativo de salvar
`txaDescricao=""` precisa de cenário BDD previamente definido pelo
🐘 Elefante na #333 e implementação separada antes de qualquer ensaio.
