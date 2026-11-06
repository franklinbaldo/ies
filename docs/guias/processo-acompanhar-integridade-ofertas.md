---
type: Guide
title: Integridade da lista e das ofertas de Acompanhamento Especial
description: Decisões de segurança do esquema de grupos, idempotência e controles inertes para a issue 254
---

# Integridade de lista e ofertas — issue #254

Detalhamento das decisões da [operação acompanhar](../decisoes/processo-acompanhar.md),
separado por responsabilidade para respeitar o limite de tamanho do projeto.

## Décima quarta fatia — esquema da lista e observação comprovada (06/11/2026, 🐢)

**Problema verificado no parser:** `acompanhamentos_da_lista` reconhecia
qualquer cabeçalho que contivesse as palavras Grupo e Observação, mas
interpretava os valores por posição fixa. Um cabeçalho invertido,
duplicado ou publicado depois dos dados poderia confundir o campo
`grupo` e induzir inclusão idempotente ou seleção de retirada sobre
identidade sem prova. Além disso, ao detectar grupo já acompanhado, a
prévia publicava `--observacao` (ou string vazia) como se fosse a
observação existente, apesar de não ter ocorrido escrita.

**Decisão:** aceitar apenas o esquema posicional de seis colunas
observado (Grupo na segunda, Observação na terceira, sem rótulos
duplicados e com cabeçalho anterior aos dados); falhar fechado com
`colunas_de_acompanhamentos_invalidas` diante de forma desconhecida.
Na idempotência, devolver a observação da linha da lista; não a
observação solicitada que não foi aplicada. Sem nova requisição,
novo POST, execução de JS ou mudança da CLI pública.

**TDD:** no RED [CI 42211745935](https://github.com/franklinbaldo/ies/actions/runs/42211745935)
dois testes de idempotência falharam (1.731 PASS); no RED
[CI 42212114534](https://github.com/franklinbaldo/ies/actions/runs/42212114534)
falharam cinco testes (os dois anteriores e três rejeições de esquema),
com 1.731 PASS e 23 ignorados anteriores. Rustfmt também pediu uma
única união de linha de teste, corrigida junto do código. GREEN e replay
devem ser aferidos no CI exato do novo head.

**Live:** NOT_RUN nesta fatia (0 GET, 0 POST, 0 escrita, 0 resíduo).
A prévia autenticada UnB anterior e seu grupo já acompanhado são
evidência histórica, não confirmação de escrita. `--deixar --confirmar`
permanece bloqueado antes da sessão/rede.


## Guarda de ação indisponível na lista (#254, 06/11/2026)

**Observação estrutural:** na lista de Acompanhamento Especial, a abertura
do cadastro depende do botão `btnAdicionar` servido. O atributo HTML
`disabled` impede o acionamento pelo navegador; um `onclick` presente
no mesmo elemento não prova uma ação oferecida.

**Decisão:** antes de derivar a URL do cadastro, recusar botão
`disabled`, `hidden` ou contido num `<fieldset disabled>`. A descendência
é verificada por identidade do nó DOM, não por regex nem pela presença de
outro elemento com ID parecido. Reutilizar `html_atributos` e a recusa
estrutural já contratada (`tela_de_acompanhamento_inesperada`); sem
regex, execução de JS, requisições adicionais ou mudança da CLI.
O teste sintético negativo tem exatamente **4 GET, 0 POST**, e o
controle habilitado conserva **5 GET, 0 POST**.

**RED anterior à implementação:** [run 42263555032](https://github.com/franklinbaldo/ies/actions/runs/42263555032),
**1 FAIL novo, 1.757 PASS, 23 ignored preexistentes**. A evidência de
GREEN depende do CI do head final; a execução não fez acesso live ao SEI.
`--deixar --confirmar` continua bloqueado antes da sessão/rede.
 
## Guarda de ancestrais inertes do Adicionar (06/11/2026, 🐢)

A oferta de cadastro é recusada quando o botão está dentro de
`fieldset disabled`, `hidden` ou `inert`, inclusive em descendência
aninhada; a decisão, os casos RED e o orçamento estão no
[relatório de evidência da oferta inerte](../validacao-live/2026-11-06-processo-acompanhar-oferta-inerte.md).
A CLI de retirada confirmada continua bloqueada.

## Oferta inerte na exclusão — continuidade da #254 (06/11/2026, 🐢)

**Evidência:** a forma de `frmGerenciarAcompanhamento`, `acaoExcluir`
e `hdnInfraItemId` foi observada na UnB; as variantes `hidden`,
`inert` e `fieldset disabled` são casos sintéticos da mesma forma.
Pelo HTML, os controles dentro de `fieldset disabled` não participam
da submissão (salvo exceção do primeiro legend, aqui recusada por
conservadorismo). O link em região inerte tampouco oferece clique.

**Escolha:** reutilizar uma única guarda de descendência por identidade
DOM na inclusão e na exclusão, com distinção entre controle de formulário
e âncora. Recusar quando o link da linha ou o oculto obrigatório não
estiver disponível; nenhum GET/POST extra, nenhuma regex ou JS executado.
O esquema de ações/IDs da UnB continua obrigatório; não se infere uma
nova capacidade de escrita. **RED** [42289429307](https://github.com/franklinbaldo/ies/actions/runs/42289429307):
2 testes funcionais novos falharam, 1.785 passaram, 23 ignorados
preexistentes. **Live da fatia: NOT_RUN, 0 HTTP, 0 escrita**; resíduos
históricos UnB 850/UFRJ 142 preservados. Retirada pública fechada.

## Script de exclusão dentro de `template` (06/11/2026, 🐢)

**Caso sintético adversarial:** a declaração `function acaoExcluir`
fica dentro de `<template><script>…</script></template>`, enquanto o
link da linha permanece ativo, fora do `template`. O parser `tl`
retorna o nó `script` na busca estrutural, mas o navegador **não
executa** scripts no conteúdo inerte de um `template`. A existência
textual do código, portanto, não comprova oferta de exclusão.

**Decisão:** a guarda única de controles passa a recusar descendentes de
`template` e é reutilizada também no reconhecimento da declaração
global do script. A recusa é `exclusao_de_acompanhamento_nao_oferecida`,
sem nova rota ou flag. A regressão de função pura recusa a oferta e a
prévia injetada exige **4 GET e 0 POST**. Nenhum JavaScript é executado
nem se usa regex. O HTML normal com script clássico fora de template
continua permitido; a retirada pública confirmada continua fechada.

## Sobrescrita da função de exclusão após a declaração (#254)

**Variação sintética adversarial:** o script contém a declaração clássica
`function acaoExcluir` com corpo idêntico ao observado na UnB, mas
há código após o fechamento da função que redefine
`window.acaoExcluir` ou atribui `null` à função. A busca anterior
validava o primeiro corpo e ignorava o restante. Um `onclick` posterior
não necessariamente invocaria a operação inspecionada.

**Decisão:** para a forma estrita observada, a extração do corpo exige
que não exista nenhum comando adicional após a chave final no mesmo
script. Se há código não observado, recusa a oferta com
`exclusao_de_acompanhamento_nao_oferecida`; não tenta interpretar nem
executar JavaScript. Testes adversariais foram publicados antes da
correção e produziram [RED 42312529474](https://github.com/franklinbaldo/ies/actions/runs/42312529474):
**2 falhas funcionais novas, 1.828 testes aprovados, 23 ignorados anteriores**.
O controle com script clássico e corpo exato continua válido; a
retirada pública confirmada permanece bloqueada antes da rede.
