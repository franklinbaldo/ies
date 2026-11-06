---
type: OperationEvidence
title: Acompanhamento especial — oferta inerte de Adicionar
description: Regressão sintética do HTML de treinamento para botão oculto e fieldset desabilitado
operation: processo.acompanhar
---

# 🐢 Oferta inerte de cadastro, issue #254

**Observação anterior:** a lista do gerenciador observada na UnB de treinamento
publica `btnAdicionar` com URL de cadastro em `onclick`. O HTML Web
também admite a inativação da oferta com o atributo `hidden` ou com um
`fieldset disabled` ancestral; esses estados foram exercitados com
HTML sintético derivado da estrutura conhecida, sem inventar rota.

**RED:** o teste novo foi publicado isoladamente em `10880db`;
[CI RED 42272651462](https://github.com/franklinbaldo/ies/actions/runs/42272651462):
**1 FAIL comportamental novo, 1.758 PASS e 23 ignored preexistentes**.
O mock recebeu uma tentativa de GET de cadastro quando o botão deveria
ser inerte. A implementação posterior inspeciona atributos com
`html_atributos` e usa a identidade dos nós DOM para conferir a
descendência do fieldset; não executa JavaScript nem usa regex.
Os testes exigem **4 GET e 0 POST**, com e sem `--confirmar`.
O controle anterior de botão habilitado preserva **5 GET, 0 POST**.

**Validação desta sessão:** compilação e testes locais NOT_RUN
(sandbox sem Rust/DNS); validação canônica pelo `ci.yml`.
O CI intermediário [42273046626](https://github.com/franklinbaldo/ies/actions/runs/42273046626)
identificou somente divergência de Rustfmt no `prek`; corrigida sem
enfraquecer testes. Confira o CI do head final da PR para GREEN.

**Live:** NOT_RUN — login 0, prévia 0 GET, confirmação 0 POST,
verificação 0 GET; **0 requisições ao SEI, 0 escritas e 0 resíduos novos**.
Os efeitos incertos históricos em UnB (ID 850) e UFRJ (ID 142) não
foram repetidos nem alterados. `--deixar --confirmar` permanece
bloqueado antes de abrir sessão/rede.

**Próxima etapa:** observar forma de aceite saneada em outro processo
elegível de treinamento, prévia atual → confirmação única sem retry →
verificação independente e registro do resíduo. Não inferir autoria
pelas leituras históricas, nem ampliar aceite sem evidência.

## Guarda de ancestrais inertes do Adicionar (06/11/2026, 🐢)

**Observado:** a UnB publica `btnAdicionar` com URL de cadastro no
`onclick`. A guarda anterior conferia `disabled` no botão e
`fieldset disabled` **somente quando o botão era filho direto**.
Com o botão dentro de `<fieldset disabled><div>…</div></fieldset>`,
o leitor concluía incorretamente que o cadastro estava disponível.
Contêineres `hidden` e `inert` também tornam inerte a oferta mesmo
que o botão interno tenha URL plausível. Os arranjos aninhados são casos
adversariais sintéticos baseados na estrutura observada.

**Decisão:** verificar toda a descendência do ancestral inerte no DOM,
por identidade de tag e sem executar JS ou procurar atributos por regex.
Recusar antes do GET de cadastro e do POST, sem novo subcódigo ou mudança
de flags. Optou-se por falhar fechado em situações de HTML não observadas,
mesmo que o navegador pudesse tratar exceções de um `legend` de modo
distinto. O botão habilitado isolado mantém o caminho original.

**RED:** CI do teste isolado [42282321667](https://github.com/franklinbaldo/ies/actions/runs/42282321667),
`1055bdc`: **1 FAIL funcional novo / 1.784 PASS / 23 ignorados
preexistentes**; falha `requisição além do roteiro` comprovou o GET
não autorizado. **GREEN:** aferir no CI do commit posterior. A bateria
nova cobre `fieldset disabled` aninhado, contêiner `hidden` e
contêiner `inert`, cada um com e sem `--confirmar`, todos com
**4 GET/0 POST**. Live nesta fatia: NOT_RUN, **0 requisições,
0 escritas, 0 resíduos novos**. A retirada confirmada segue bloqueada.

## Retirada com oferta inerte (06/11/2026, 🐢)

A tela observada de Acompanhamento Especial na UnB contém link
`acaoExcluir` na linha do grupo e campo `hdnInfraItemId` no form de
gerenciamento. A semântica de `hidden`/`inert` e `fieldset disabled`
foi exercitada com **HTML sintético**; não se afirma que a UnB tenha
servido essas variantes ao vivo. Link oculto, ancestral `inert` e
`input hidden` dentro de `fieldset disabled` eram indevidamente
interpretados como oferta de retirada. A guarda agora examina a
identidade do controle e todos os ancestrais, distinguindo âncoras de
campos enviáveis. Sem regex nem JS executado.

**RED** [CI 42289429307](https://github.com/franklinbaldo/ies/actions/runs/42289429307):
**2 testes novos falharam, 1.785 passaram, 23 ignorados existentes**;
o Rustfmt do próprio teste também exigiu dois ajustes de apresentação.
A sequência foi: teste primeiro, CI vermelho, código/extração compartilhada
em seguida. Teste puro, **0 GET e 0 POST**, sem mexer na contagem
já contratada dos fluxos de transporte. GREEN final: conferir head
exato da PR #293.

**Live desta fatia: NOT_RUN**, prévia 0 GET, confirmação 0 POST,
releitura 0 GET, 0 requisições SEI, 0 escritas e 0 novos resíduos.
IDs anteriores UnB 850 / UFRJ 142 continuam inconclusivos e não
foram alterados. `--deixar --confirmar` continua fechado antes de
sessão/rede. Próxima etapa: ensaio seguro no treinamento em novo NUP.

## Função `acaoExcluir` inerte sob `template` (06/11/2026, 🐢)

O HTML observado da UnB contém a função global clássica e o acionador da
linha. A variante `<template><script>…</script></template>` é
**sintética**, derivada dessa tela, sem afirmar observação live.
A revisão 🦅 do head `b837e4b` detectou que a busca por `script`
aceitava uma declaração dentro de `template` como oferta executável.

**RED antes do código:** os testes foram publicados em `7fd5169`.
No [CI 42306561651](https://github.com/franklinbaldo/ies/actions/runs/42306561651),
**2 testes novos FAIL, 1.826 PASS, 23 ignored preexistentes**:
o reconhecedor puro aceitou o script inerte e a prévia deixou de recusar
a oferta. A correção do código está em `fd746d4`, reutilizando
`controle_inerte` para excluir scripts e controles dentro de
`template`, sem executar JS/regex, retry nem criar nova rota.
O teste de prévia exige **4 GET, 0 POST**; o teste de função pura não
realiza rede. Conferir o GREEN no CI do head final.

**Sincronização:** `main@04514cf` integrada em merge de dois pais
`e73fb06` antes do RED, preservando as entradas documentais das
issues #254 e #271. Não houve rebase nem force-push.

**Live desta rodada:** **NOT_RUN**, 0 requisições reais (0 GET,
0 POST), 0 escritas, 0 novos resíduos. A retirada confirmada segue
bloqueada antes de sessão/rede. Os resíduos incertos anteriores
UnB 850 e UFRJ 142 não foram repetidos nem alterados.
