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

Os cards foram lidos; os membros ZIP foram conferidos no job de inspeção abaixo, mas não houve acesso local aos corpos originais. O Todos documenta link da linha para alteração do bloco, formulário `frmBlocoCadastro`, `hdnIdBloco`, lista `selUnidades`, autocomplete ligado a `txtUnidade`/`hdnIdUnidade` e ação `unidade_auto_completar_outras`. A fixture do Todos usa **POST consultivo** para o autocomplete; não confundir com mutação de cadastro, nem presumir GET. Confirmar método e resposta originais antes de alegar compatibilidade.

## Resultado da inspeção (27/11/2026, R300)

[Job 140788394345](https://github.com/franklinbaldo/ies/actions/runs/46083728588/job/140788394345): SHA-1 dos ZIPs e SHA-256 LF dos membros conferidos, schema 1 completo, 87 interações. A inspeção conta métodos e ações, não interpreta os corpos.

| Instalação | GET alterar | POST alterar | POST autocomplete |
| --- | ---: | ---: | ---: |
| PEN3 | 3 | 1 | 2 |
| UnB | 6 | 2 | 2 |

A ação de autocomplete é um POST nas duas instalações. O teste Todos `test_bloco_alterar_010.py` marca `sem_cassette` e usa XML sintético `<itens><item id="..." descricao="..."/></itens>`; não trata esse formato como testemunho primário. A [resposta da Lontra](https://github.com/franklinbaldo/ies/issues/876#issuecomment-7230447903) não localizou replay nominal dessa ação; não substituir pelo replay GO da lupa.

### Delta mínimo identificado no IES

- `src/acao_do_sei.rs`: faltam as duas variantes nominais `bloco_assinatura_alterar` e `unidade_auto_completar_outras`. O enum AJAX já possui `unidade_auto_completar_envio_processo`, ação diferente.
- `src/bloco.rs`: o leitor preserva o link do número para `rel_bloco_protocolo_listar`, não a ação de alteração na mesma linha. Extrair apenas o link de alteração publicado para o bloco pedido; não reaproveitar a URL de detalhe como formulário de edição.
- `src/formulario_web/ajax.rs`: derivação de consulta já confere instalação, ação e hash; reutilizar sem aceitar URLs inventadas. Os campos exatos e a representação da resposta da nova ação ainda precisam ser confrontados com os originais, não copiados do fluxo de envio de processo.
- `selUnidades` ausente não equivale a lista vazia. A fixture do Todos distingue select vazio, ausente e duplicado. O no-op usa identidade da unidade, não igualdade de rótulo. Prévia não deve serializar `hdnUnidades` nem enviar formulário de cadastro.

Próximo replay: parear cada GET alterar com o número publicado e a identidade oculta do formulário; conferir os dois POSTs de autocomplete por instalação (método, campos e formato do retorno), comparar a seleção por ID com as opções existentes e incluir uma negativa sem POST mutador. Não imprimir URLs assinadas, corpos privados ou tokens. A execução Rust/E2E e o replay original seguem NOT_RUN; a aprovação da sintaxe segue pendente.

## Aceite da implementação futura

1. Reutilizar leitura de lista e transporte IES; seguir o link publicado para o bloco exato e confrontar a identidade do formulário. Não construir URLs nem escolher bloco por descrição.
2. Ler destinos atuais e resolver uma unidade pelo autocomplete publicado na tela, usando o método observado; ausência ou ambiguidade deve falhar sem escolher a primeira sugestão.
3. Prévia deve aceitar caso positivo e no-op; recusar ação ausente, formulário de outro bloco, zero correspondências e correspondências ambíguas. Provar zero POST **mutador** em todos os casos, inclusive tentativa de confirmação. POST consultivo não comprova escrita nem deve ser proibido por contagem genérica.
4. E2E CLI real em loopback: confrontar resultado com respostas originais, usar estado inicial independente quando aplicável, mostrar que destinos anteriores não mudaram, sem alteração SQLite/migration. Não confundir inspeção de metadados com replay.
5. Depois de revisão do contrato, implementação mínima em branch própria e CI/replays no SHA final. Escrita futura exige outra fatia, releitura independente e revisão excepcional.

Nesta proposta não há código produtivo nem execução do binário. O pedido temporário de inspeção é descartável após a coleta; não faz parte da capacidade.
