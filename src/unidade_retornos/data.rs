#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MesAno {
    ano: u16,
    mes: u8,
}

impl MesAno {
    pub(crate) fn new(ano: u16, mes: u8) -> Option<Self> {
        (ano > 0 && (1..=12).contains(&mes)).then_some(Self { ano, mes })
    }

    pub(crate) fn ano(self) -> u16 {
        self.ano
    }

    pub(crate) fn mes(self) -> u8 {
        self.mes
    }

    pub(crate) fn ler_cli(s: &str) -> Option<Self> {
        let (a, m) = s.split_once('-')?;
        if s.len() != 7
            || !a.bytes().all(|b| b.is_ascii_digit())
            || !m.bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }
        Self::new(a.parse().ok()?, m.parse().ok()?)
    }

    pub(crate) fn ultimo_dia(self) -> u8 {
        let bissexto = self.ano.is_multiple_of(4)
            && (!self.ano.is_multiple_of(100) || self.ano.is_multiple_of(400));
        match self.mes {
            2 if bissexto => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }

    pub(crate) fn vizinho(self, rumo: i32) -> Option<Self> {
        match (self.mes, rumo) {
            (12, 1) => Self::new(self.ano.checked_add(1)?, 1),
            (1, -1) => Self::new(self.ano.checked_sub(1)?, 12),
            (1..=11, 1) => Self::new(self.ano, self.mes + 1),
            (2..=12, -1) => Self::new(self.ano, self.mes - 1),
            _ => None,
        }
    }

    pub(crate) fn posicao(self) -> i32 {
        i32::from(self.ano) * 12 + i32::from(self.mes)
    }
}

impl std::fmt::Display for MesAno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}", self.ano, self.mes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DataCivil {
    mes_ano: MesAno,
    dia: u8,
}

impl DataCivil {
    pub(crate) fn new(ano: u16, mes: u8, dia: u8) -> Option<Self> {
        let mes_ano = MesAno::new(ano, mes)?;
        (1..=mes_ano.ultimo_dia())
            .contains(&dia)
            .then_some(Self { mes_ano, dia })
    }

    pub(crate) fn mes_ano(self) -> MesAno {
        self.mes_ano
    }

    pub(crate) fn dia(self) -> u8 {
        self.dia
    }

    pub(crate) fn ler(s: &str) -> Option<Self> {
        let partes: Vec<_> = s.split('/').collect();
        let [dia, mes, ano] = partes.as_slice() else {
            return None;
        };
        if dia.len() != 2
            || mes.len() != 2
            || ano.len() != 4
            || !s.bytes().all(|b| b.is_ascii_digit() || b == b'/')
        {
            return None;
        }
        Self::new(ano.parse().ok()?, mes.parse().ok()?, dia.parse().ok()?)
    }
}

impl std::fmt::Display for DataCivil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}/{:02}/{:04}",
            self.dia, self.mes_ano.mes, self.mes_ano.ano
        )
    }
}
