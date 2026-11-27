---
type: Guide
title: "Proposta: prévia de destino de bloco de assinatura"
description: "Recorte consultivo de #876 para deliberação, sem alteração do contrato ativo"
---

# Proposta focal — #876, R299

**Status: proposta, não capacidade integrada.** A [ordem R299](https://github.com/franklinbaldo/ies/issues/876#issuecomment-7229887298) separa a prévia do POST mutador, da #905 e das migrations. Não modifica a CLI contratada.

## Interface proposta

`ies bloco alterar BLOCO --unidade UNIDADE`, alinhada à sintaxe do Todos, inicialmente apenas prévia. Não oferecer confirmação nesta fatia. `--unidade` deve acrescentar a unidade resolvida, nunca substituir as existentes. Se já associada pelo mesmo ID, mostrar no-op. Não acoplar disponibilizar, devolver, concluir, alteração de descrição/grupo ou remoção.

Saída deve identificar o número do bloco, unidade resolvida (ID e nome), se já pertence ao cadastro e se haveria mudança; JSON e saída humana não afirmam escrita. Sem expor URLs assinadas ou tokens.

## Autoridade das fontes

- [PEN3](../cassettes/pen3/bloco-disponibilizar-pen3--aec2a85f490e.md): 56 interações, 4 alterar, 2 autocomplete.
- [UnB](../cassettes/unb/bloco-alterar-unb--d36a8c8725eb.md): 31 interações, 8 alterar, 2 autocomplete.
- [Todos: descrição observacional](https://github.com/franklinbaldo/todos/blob/main/docs/como-o-sei-funciona/15-telas-da-barra/concluir-reabrir-e-disponibilizar-o-bloco.md), blob `d0dfaa9480ab18d5ce6b9eebe26d715867c3c5c8`.
- [Todos: regressões](https://github.com/franklinbaldo/todos/blob/main/tests/test_bloco_alterar_010.py), blob `327b6ad38bcd07b363b0261febb763bca4af8b24`. A fixture combina lista UFRJ e formulário sintético baseado em UnB; não é prova original de PEN3.

Os cards foram lidos, não os membros ZIP (DNS indisponível). O Todos documenta link da linha para alteração do bloco, formulário `frmBlocoCadastro`, `hdnIdBloco`, lista `selUnidades`, autocomplete ligado a `txtUnidade`/`hdnIdUnidade` e ação `unidade_auto_completar_outras`. A fixture do Todos usa **POST consultivo** para o autocomplete; não confundir com mutação de cadastro, nem presumir GET. Confirmar método e resposta originais antes de alegar compatibilidade.

## Aceite da implementação futura

1. Reutilizar leitura de lista e transporte IES; seguir o link publicado para o bloco exato e confrontar a identidade do formulário. Não construir URLs nem escolher bloco por descrição.
2. Ler destinos atuais e resolver uma unidade pelo autocomplete publicado na tela, usando o método observado; ausência ou ambiguidade deve falhar sem escolher a primeira sugestão.
3. Prévia deve aceitar caso positivo e no-op; recusar ação ausente, formulário de outro bloco, zero correspondências e correspondências ambíguas. Provar zero POST **mutador** em todos os casos, inclusive tentativa de confirmação. POST consultivo não comprova escrita nem deve ser proibido por contagem genérica.
4. E2E CLI real em loopback: confrontar resultado com respostas originais, usar estado inicial independente quando aplicável, mostrar que destinos anteriores não mudaram, sem alteração SQLite/migration. Não confundir inspeção de metadados com replay.
5. Depois de revisão do contrato, implementação mínima em branch própria e CI/replays no SHA final. Escrita futura exige outra fatia, releitura independente e revisão excepcional.

Nesta proposta não há código produtivo nem execução do binário. O pedido temporário de inspeção deve ser removido da branch antes do aceite.
