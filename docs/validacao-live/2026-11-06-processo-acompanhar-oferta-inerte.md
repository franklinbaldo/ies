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
